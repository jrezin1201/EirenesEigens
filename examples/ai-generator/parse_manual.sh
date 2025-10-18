#!/bin/bash

# Manual AI Generator Parser
# Extracts files from JSON response you got from an LLM

if [ -z "$1" ]; then
    echo "Usage: ./parse_manual.sh <project-name>"
    echo ""
    echo "Example:"
    echo "  ./parse_manual.sh hawaiian-shirts"
    echo ""
    echo "This will look for: generated/hawaiian-shirts/response.json"
    exit 1
fi

PROJECT_NAME=$1
PROJECT_DIR="generated/$PROJECT_NAME"
RESPONSE_FILE="$PROJECT_DIR/response.json"

echo "🔍 Looking for JSON response..."

# Check if response file exists
if [ ! -f "$RESPONSE_FILE" ]; then
    echo "❌ Error: $RESPONSE_FILE not found"
    echo ""
    echo "Please create the file first:"
    echo "  mkdir -p $PROJECT_DIR"
    echo "  nano $RESPONSE_FILE"
    echo "  (paste your JSON, then Ctrl+X, Y, Enter)"
    exit 1
fi

echo "✅ Found: $RESPONSE_FILE"
echo ""
echo "📝 Extracting files..."
echo ""

# Extract and create each file
cat "$RESPONSE_FILE" | jq -r '.files[] | @json' | while read -r file; do
    FILE_PATH=$(echo "$file" | jq -r '.path')
    FILE_CONTENT=$(echo "$file" | jq -r '.content')

    # Create directory if needed
    mkdir -p "$PROJECT_DIR/$(dirname "$FILE_PATH")"

    # Write file
    echo "$FILE_CONTENT" > "$PROJECT_DIR/$FILE_PATH"

    # Count lines for feedback
    LINE_COUNT=$(echo "$FILE_CONTENT" | wc -l | tr -d ' ')

    echo "✅ Created: $FILE_PATH ($LINE_COUNT lines)"
done

echo ""
echo "🎉 Done! Files extracted to: $PROJECT_DIR"
echo ""
echo "📁 Project structure:"
cd "$PROJECT_DIR"
find . -type f -not -path "*/\.*" | sort

echo ""
echo "🚀 Next steps:"
echo "  cd $PROJECT_DIR"
echo "  cat src/main.raven    # Review the code"
echo "  mkdir -p dist"
echo "  raven compile src/main.raven --output dist/app.wasm"
echo ""
