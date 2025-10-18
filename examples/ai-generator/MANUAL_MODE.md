# Manual AI Generator Mode

Use this when you want to generate the code yourself using Claude.ai or any other LLM, then paste it into RavensOne.

## Step 1: Copy This Prompt

Use this prompt in Claude.ai, ChatGPT, or any LLM:

```
You are an expert RavensOne developer. RavensOne is a reactive web framework with a Rust-like syntax.

# RavensOne Language Specification

## Component Structure
```raven
component ComponentName(props: Props) {
    // State declarations
    let state_var = Signal::new(initial_value);
    let computed_var = Computed::new(|| state_var.get() * 2);

    // Event handlers
    fn handle_event() {
        state_var.set(new_value);
    }

    // Template
    <div class="container">
        <h1>{computed_var.get()}</h1>
        <button onclick={handle_event}>Click</button>
    </div>
}
```

## Reactive Primitives
- Signal::new(value) - Reactive state
- Computed::new(fn) - Derived state
- Effect::new(fn) - Side effects

## Your Task

Create a RavensOne project with these details:

**Project Name**: [YOUR PROJECT NAME]
**Description**: [YOUR DESCRIPTION]
**Features**: [YOUR FEATURES]

Return ONLY valid JSON with this exact structure:
{
  "files": [
    {"path": "src/main.raven", "content": "...full raven code here..."},
    {"path": "raven.toml", "content": "...toml content..."}
  ],
  "dependencies": []
}

Return ONLY the JSON, no markdown code blocks, no explanations.
```

## Step 2: Replace the Placeholders

In the prompt above, replace:
- `[YOUR PROJECT NAME]` with your project name (e.g., "hawaiian-shirts")
- `[YOUR DESCRIPTION]` with what you want (e.g., "E-commerce store for Hawaiian shirts")
- `[YOUR FEATURES]` with features (e.g., "product grid with images, size selector, add to cart, price display")

## Step 3: Get the JSON Response

Paste the modified prompt into Claude.ai (https://claude.ai) or your preferred LLM.

The LLM should return JSON like this:

```json
{
  "files": [
    {
      "path": "src/main.raven",
      "content": "use raven::prelude::*;\n\ncomponent App() {\n  ...\n}"
    },
    {
      "path": "raven.toml",
      "content": "[package]\nname = \"my-app\"\n..."
    }
  ],
  "dependencies": []
}
```

## Step 4: Save the JSON

Copy the entire JSON response and save it to a file:

```bash
cd examples/ai-generator
mkdir -p generated/my-project
# Paste the JSON into a file:
nano generated/my-project/response.json
# (paste the JSON, then Ctrl+X, Y, Enter to save)
```

## Step 5: Run the Parser Script

I'll create a script that extracts the files from your JSON:

```bash
./parse_manual.sh my-project
```

This will:
- Read `generated/my-project/response.json`
- Extract each file
- Create the proper directory structure
- Save all files

## Step 6: Compile and Run

```bash
cd generated/my-project
cat src/main.raven  # Verify it looks good
```

---

## Quick Example

### Your Prompt to LLM:
```
Create a RavensOne project:
Name: counter
Description: A simple counter with buttons
Features: increment button, decrement button, reset button, keyboard shortcuts

[Include the full system prompt from Step 1]
```

### You'll Get Back:
```json
{
  "files": [
    {
      "path": "src/main.raven",
      "content": "component App() { let count = Signal::new(0); ... }"
    },
    ...
  ]
}
```

### Then Run:
```bash
# Save JSON to generated/counter/response.json
./parse_manual.sh counter
cd generated/counter
cat src/main.raven
```

---

## Tips

**Keep it Simple**: Start with 3-5 features, not 30+

**Good Examples**:
- "Counter with increment/decrement/reset"
- "Todo list with add/delete/filter"
- "Product grid with images and prices"

**Be Specific**:
- Instead of "looks nice", say "gradient background, rounded corners"
- Instead of "cart", say "shopping cart with quantity adjustment"

---

## Hawaiian Shirts Example

Here's exactly what to paste into Claude.ai:

```
You are an expert RavensOne developer. Create a Hawaiian shirt e-commerce store.

Project Name: hawaiian-shirts
Description: E-commerce product grid for Hawaiian shirts
Features: product cards with images, product titles, price display, size selector dropdown, color filter buttons, add to cart button, cart icon with count

Use RavensOne reactive syntax with Signals for state.
Include beautiful styling with a tropical color scheme.

Return ONLY valid JSON with files array containing src/main.raven and raven.toml.
No markdown code blocks, just pure JSON.
```

Then save the response and run the parser!
