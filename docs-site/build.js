// Simple script to convert markdown docs to HTML
const fs = require('fs');
const path = require('path');

const template = (title, content) => `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>${title} - RavensOne Documentation</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
        }
        nav {
            background: white;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            position: sticky;
            top: 0;
            z-index: 100;
        }
        nav .container {
            max-width: 1200px;
            margin: 0 auto;
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1rem 2rem;
        }
        nav a {
            color: #333;
            text-decoration: none;
            margin: 0 1rem;
            transition: color 0.2s;
        }
        nav a:hover { color: #667eea; }
        .logo {
            font-weight: bold;
            font-size: 1.5rem;
            color: #667eea;
        }
        .container {
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
            background: white;
            margin-top: 2rem;
            margin-bottom: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 { color: #667eea; margin: 2rem 0 1rem; font-size: 2.5rem; }
        h2 { color: #667eea; margin: 2rem 0 1rem; border-bottom: 2px solid #667eea; padding-bottom: 0.5rem; }
        h3 { color: #764ba2; margin: 1.5rem 0 0.5rem; }
        code {
            background: #f4f4f4;
            padding: 0.2rem 0.4rem;
            border-radius: 3px;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.9em;
        }
        pre {
            background: #2d2d2d;
            color: #f8f8f2;
            padding: 1.5rem;
            border-radius: 8px;
            overflow-x: auto;
            margin: 1rem 0;
        }
        pre code {
            background: none;
            padding: 0;
            color: inherit;
        }
        a { color: #667eea; text-decoration: none; }
        a:hover { text-decoration: underline; }
        ul, ol { margin-left: 2rem; margin-bottom: 1rem; }
        li { margin: 0.5rem 0; }
        footer {
            background: #2d2d2d;
            color: white;
            text-align: center;
            padding: 2rem;
            margin-top: 4rem;
        }
    </style>
</head>
<body>
    <nav>
        <div class="container" style="max-width: none;">
            <div class="logo">RavensOne</div>
            <div>
                <a href="/">Home</a>
                <a href="/getting-started.html">Get Started</a>
                <a href="https://github.com/jrezin1201/RavensOne">GitHub</a>
            </div>
        </div>
    </nav>

    <div class="container">
        ${content}
    </div>

    <footer>
        <p>&copy; 2025 RavensOne. MIT License.</p>
        <p style="margin-top: 0.5rem;">
            <a href="https://github.com/jrezin1201/RavensOne" style="color: #667eea;">GitHub</a> •
            <a href="/getting-started.html" style="color: #667eea;">Documentation</a>
        </p>
    </footer>
</body>
</html>`;

// Simple markdown to HTML converter
function mdToHtml(md) {
    return md
        // Headers
        .replace(/^### (.+)$/gm, '<h3>$1</h3>')
        .replace(/^## (.+)$/gm, '<h2>$1</h2>')
        .replace(/^# (.+)$/gm, '<h1>$1</h1>')
        // Code blocks
        .replace(/```(\w+)?\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>')
        // Inline code
        .replace(/`([^`]+)`/g, '<code>$1</code>')
        // Bold
        .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
        // Links
        .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
        // Lists
        .replace(/^- (.+)$/gm, '<li>$1</li>')
        // Paragraphs (simple)
        .replace(/\n\n/g, '</p><p>')
        // Wrap lists
        .replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>')
        // Clean up
        .replace(/<\/li>\n<li>/g, '</li><li>')
        ;
}

// Read Getting Started markdown
const gettingStartedMd = fs.readFileSync(path.join(__dirname, '../docs/GETTING_STARTED.md'), 'utf8');
const gettingStartedHtml = template('Getting Started', mdToHtml(gettingStartedMd));
fs.writeFileSync(path.join(__dirname, 'public/getting-started.html'), gettingStartedHtml);

console.log('✅ Documentation site built successfully!');
console.log('📂 Output: docs-site/public/');
