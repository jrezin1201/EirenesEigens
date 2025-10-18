use std::fs;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub project_description: String,
    pub project_name: String,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedProject {
    pub files: Vec<GeneratedFile>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

pub struct AIGenerator {
    api_key: String,
}

impl AIGenerator {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Generate the system prompt that guides Claude to create RavensOne code
    pub fn create_system_prompt() -> String {
        r#"You are an expert RavensOne developer. RavensOne is a reactive web framework with a Rust-like syntax.

# RavensOne Language Specification

## Component Structure
```raven
component ComponentName(props: Props) {
    // State declarations
    let state_var = Signal::new(initial_value);
    let computed_var = Computed::new(|| state_var.get() * 2);

    // Effects
    Effect::new(|| {
        // Side effects that run when dependencies change
    });

    // Event handlers
    fn handle_event() {
        state_var.set(new_value);
    }

    // Template (JSX-like syntax)
    <div class="container">
        <h1>{computed_var.get()}</h1>
        <button onclick={handle_event}>Click</button>
    </div>
}
```

## Reactive Primitives

### Signal - Reactive State
```raven
let count = Signal::new(0);
count.set(count.get() + 1);
```

### Computed - Derived State
```raven
let doubled = Computed::new(|| count.get() * 2);
let value = doubled.get();
```

### Effect - Side Effects
```raven
Effect::new(|| {
    console::log(&format!("Count: {}", count.get()));
});
```

## Built-in Components

### Form Inputs
```raven
<Input
    value={name.get()}
    oninput={|e| name.set(e.target.value)}
    placeholder="Enter name"
/>
```

### Conditional Rendering
```raven
{if show.get() {
    <div>Visible content</div>
} else {
    <div>Hidden content</div>
}}
```

### Lists
```raven
{items.get().iter().map(|item| {
    <li key={item.id}>{item.name}</li>
}).collect::<Vec<_>>()}
```

## API Integration
```raven
async fn fetch_data() -> Result<Data, Error> {
    let response = http::get("/api/endpoint").await?;
    response.json().await
}
```

## Your Task
When the user describes a project, you should:

1. **Create the main component** in `src/main.raven`
2. **Create sub-components** in `src/components/*.raven` as needed
3. **Create a `raven.toml`** manifest file
4. **Use proper reactive patterns** (Signals, Computed, Effects)
5. **Include proper styling** with CSS classes
6. **Add comments** explaining the code

## Output Format
Return ONLY valid JSON in this exact format:
```json
{
  "files": [
    {
      "path": "src/main.raven",
      "content": "component App() { ... }"
    },
    {
      "path": "src/components/TodoItem.raven",
      "content": "component TodoItem(props: TodoItemProps) { ... }"
    },
    {
      "path": "raven.toml",
      "content": "[package]\nname = \"...\"\n..."
    }
  ],
  "dependencies": ["raven-ui", "raven-router"]
}
```

IMPORTANT:
- Return ONLY the JSON, no markdown code blocks
- Use proper RavensOne syntax
- Include all necessary files
- Make the code production-ready"#.to_string()
    }

    /// Create the user prompt from the generation request
    pub fn create_user_prompt(request: &GenerationRequest) -> String {
        format!(
            r#"Create a RavensOne project with the following specifications:

Project Name: {}
Description: {}
Features: {}

Generate all necessary .raven files for this project. Include:
- Main application component
- Any sub-components needed
- Proper state management with Signals
- Event handlers
- Styling with CSS classes
- The raven.toml manifest

Return the complete project structure in JSON format."#,
            request.project_name,
            request.project_description,
            request.features.join(", ")
        )
    }

    /// Call Claude API to generate the project
    pub async fn generate_project(&self, request: &GenerationRequest) -> Result<GeneratedProject, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let system_prompt = Self::create_system_prompt();
        let user_prompt = Self::create_user_prompt(request);

        #[derive(Serialize)]
        struct ClaudeRequest {
            model: String,
            max_tokens: u32,
            system: String,
            messages: Vec<Message>,
        }

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct ClaudeResponse {
            content: Vec<Content>,
        }

        #[derive(Deserialize)]
        struct Content {
            text: String,
        }

        let request_body = ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 8000,
            system: system_prompt,
            messages: vec![Message {
                role: "user".to_string(),
                content: user_prompt,
            }],
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let claude_response: ClaudeResponse = response.json().await?;
        let generated_text = &claude_response.content[0].text;

        // Parse the JSON response from Claude
        let project: GeneratedProject = serde_json::from_str(generated_text)?;

        Ok(project)
    }

    /// Write the generated files to disk
    pub fn write_project_files(&self, project: &GeneratedProject, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory
        fs::create_dir_all(output_dir)?;

        // Write each file
        for file in &project.files {
            let file_path = output_dir.join(&file.path);

            // Create parent directories if needed
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&file_path, &file.content)?;
            println!("✅ Created: {}", file.path);
        }

        Ok(())
    }

    /// Compile the generated RavensOne project
    pub fn compile_project(&self, project_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔨 Compiling RavensOne project...");

        let output = Command::new("raven")
            .arg("compile")
            .arg(project_dir.join("src/main.raven"))
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Compilation failed: {}", error).into());
        }

        println!("✅ Compilation successful!");
        Ok(())
    }

    /// Full workflow: generate, write, and compile
    pub async fn generate_and_compile(&self, request: &GenerationRequest, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Generating project with Claude...");
        let project = self.generate_project(request).await?;

        println!("📝 Writing {} files...", project.files.len());
        self.write_project_files(&project, output_dir)?;

        println!("🔨 Compiling project...");
        self.compile_project(output_dir)?;

        println!("🎉 Project generated and compiled successfully!");
        println!("📁 Output directory: {}", output_dir.display());

        Ok(())
    }
}
