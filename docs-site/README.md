# RavensOne Documentation Site

Modern, responsive documentation site for the RavensOne web framework.

## 🌐 Live Site

**https://ravensone-docs.fly.dev**

## 📁 Structure

```
docs-site/
├── index.html              # Landing page
├── css/
│   ├── style.css          # Main stylesheet
│   └── docs.css           # Documentation-specific styles
├── js/
│   └── main.js            # Interactive features
├── pages/
│   ├── getting-started.html  # Getting Started guide
│   ├── docs.html            # Full documentation
│   └── packages.html        # Package manager docs
├── Dockerfile             # Container configuration
├── nginx.conf             # Web server config
└── fly.toml               # Fly.io deployment config
```

## 🚀 Deployment

Deploy to Fly.io:
```bash
flyctl deploy --app ravensone-docs
```

## 💰 Cost

- **$0/month** (Free tier)
- Auto-stop/start machines
- 20MB image size
