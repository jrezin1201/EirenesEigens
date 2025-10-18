#!/bin/bash

# RavensOne AI Generator - Interactive Script
# Generates RavensOne projects from natural language descriptions using Claude

set -e

echo "🤖 RavensOne AI Project Generator"
echo "=================================="
echo ""

# Check for API key
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "❌ Error: ANTHROPIC_API_KEY environment variable not set"
    echo "Set it with: export ANTHROPIC_API_KEY='your-api-key'"
    exit 1
fi

# Get project details from user
read -p "Project name: " PROJECT_NAME
read -p "Project description: " PROJECT_DESC
read -p "Features (comma-separated): " FEATURES

# Create output directory
OUTPUT_DIR="generated/$PROJECT_NAME"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "🤖 Generating project with Claude..."
echo ""

# Create the prompt
SYSTEM_PROMPT='You are an expert RavensOne developer. RavensOne is a reactive web framework with a Rust-like syntax.

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

Return ONLY valid JSON with this structure:
{
  "files": [
    {"path": "src/main.raven", "content": "..."},
    {"path": "raven.toml", "content": "..."}
  ],
  "dependencies": []
}'

USER_PROMPT="Create a RavensOne project:
Name: $PROJECT_NAME
Description: $PROJECT_DESC
Features: $FEATURES

Generate all .raven files needed. Return ONLY the JSON, no markdown."

# Call Claude API
RESPONSE=$(curl -s https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d "{
    \"model\": \"claude-sonnet-4-20250514\",
    \"max_tokens\": 8000,
    \"system\": $(echo "$SYSTEM_PROMPT" | jq -R -s .),
    \"messages\": [{
      \"role\": \"user\",
      \"content\": $(echo "$USER_PROMPT" | jq -R -s .)
    }]
  }")

# Extract the generated code from Claude's response
GENERATED_JSON=$(echo "$RESPONSE" | jq -r '.content[0].text')

# Save the raw response for debugging
echo "$GENERATED_JSON" > "$OUTPUT_DIR/generated.json"

# Extract and write each file
echo "$GENERATED_JSON" | jq -r '.files[] | @json' | while read -r file; do
    FILE_PATH=$(echo "$file" | jq -r '.path')
    FILE_CONTENT=$(echo "$file" | jq -r '.content')

    # Create directory if needed
    mkdir -p "$OUTPUT_DIR/$(dirname "$FILE_PATH")"

    # Write file
    echo "$FILE_CONTENT" > "$OUTPUT_DIR/$FILE_PATH"
    echo "✅ Created: $FILE_PATH"
done

echo ""
echo "🔨 Compiling project..."
cd "$OUTPUT_DIR"

# Compile with RavensOne compiler
if [ -f "src/main.raven" ]; then
    raven compile src/main.raven --output dist/app.wasm
    echo "✅ Compilation successful!"
else
    echo "⚠️  No main.raven file found, skipping compilation"
fi

echo ""
echo "🎉 Project generated successfully!"
echo "📁 Location: $OUTPUT_DIR"
echo ""
echo "Next steps:"
echo "  cd $OUTPUT_DIR"
echo "  raven dev    # Start development server"
echo ""
