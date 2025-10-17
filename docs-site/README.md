# RavensOne Documentation Site

Official documentation website for RavensOne - the AI-first full-stack web framework.

## 🚀 Quick Start

### Build the site

```bash
node build.js
```

This will convert the markdown documentation in `../docs/` to HTML and output to `public/`.

### Deploy to Vercel

```bash
# Install Vercel CLI if not already installed
npm install -g vercel

# Deploy
vercel --prod
```

Or connect your GitHub repository to Vercel for automatic deployments.

## 📁 Structure

```
docs-site/
├── public/              # Static HTML files (generated)
│   ├── index.html       # Homepage
│   └── getting-started.html  # Getting Started guide
├── build.js             # Build script (converts MD → HTML)
├── vercel.json          # Vercel deployment config
└── README.md            # This file
```

## 🔧 Development

To preview the site locally:

```bash
# Simple Python server
python3 -m http.server 8000 --directory public

# Or use Node.js
npx serve public
```

Then visit `http://localhost:8000`

## 📝 Adding New Pages

1. Create markdown file in `../docs/`
2. Update `build.js` to convert the new markdown file
3. Run `node build.js`
4. Deploy to Vercel

## 🎨 Styling

All styles are inline in the HTML template in `build.js`. To update the theme:

1. Edit the `template()` function in `build.js`
2. Rebuild with `node build.js`

## 🌐 Live Site

Once deployed, the site will be available at:
- Production: https://ravensone.vercel.app (or your custom domain)
- Preview: Vercel will generate preview URLs for each deployment

## 📄 License

MIT License - see LICENSE file in the main repository
