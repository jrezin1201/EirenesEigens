# RavensOne Analytics Dashboard - Example Application

A production-ready analytics dashboard showcasing all RavensOne features: reactive state management, SSR, type-safe full-stack development, and progressive hydration.

## 🚀 Quick Start

### Run Locally (Instant)

```bash
# Open directly in browser
open run_dashboard.html

# OR use a local server
npm run dev
```

Visit: `http://localhost:8000/run_dashboard.html`

### Deploy to Vercel (2 minutes)

```bash
# Install Vercel CLI
npm install -g vercel

# Deploy
cd examples
vercel

# Deploy to production
vercel --prod
```

**That's it!** Your app is now live at `https://your-project.vercel.app`

## 📋 What's Included

- **`analytics_dashboard.raven`** - Source code in RavensOne language
- **`run_dashboard.html`** - Interactive demo (works immediately!)
- **`analytics_dashboard.css`** - Professional styling
- **`vercel.json`** - Vercel deployment configuration
- **`package.json`** - NPM configuration
- **`DEPLOYMENT_GUIDE.md`** - Complete deployment instructions
- **`ANALYTICS_README.md`** - Architecture documentation
- **`HOW_TO_RUN.md`** - Detailed running instructions

## ✨ Features Demonstrated

### Reactive State Management
```javascript
const count = Signal.new(0);
const double = Computed.new(() => count.get() * 2);
create_effect(() => console.log(count.get()));
```

### Server-Side Rendering (SSR)
```rust
render_to_document(vnode, ctx, app_name) -> HTML
```

### Progressive Hydration
```rust
HydrationStrategy::Immediate  // Critical components
HydrationStrategy::WhenVisible  // Lazy-load on scroll
```

### Type-Safe Full Stack
```rust
extern server fn fetch_metrics(user_id: String) -> Vec<Metric>
```

## 🎯 Live Demo Features

- ✅ 4 real-time metrics (Revenue, Users, Conversion, Session Duration)
- ✅ Interactive date range selection
- ✅ 30-day trend charts with Canvas API
- ✅ Smooth loading states
- ✅ Reactive updates (change propagation)
- ✅ Mobile responsive design
- ✅ Production-grade styling

## 📊 Performance

Expected metrics on Vercel:
- **First Paint**: < 100ms
- **Time to Interactive**: < 200ms
- **Lighthouse Score**: 95-100
- **Bundle Size**: 0KB (inline JS, future: ~50KB WASM)

## 🛠️ Customization

### Change Metrics

Edit `run_dashboard.html`, modify `mockMetrics`:

```javascript
const mockMetrics = [
    {
        id: 'm1',
        name: 'Your Custom Metric',
        value: 12345,
        change_percent: 15.5
    }
];
```

### Update Styling

Edit `analytics_dashboard.css`:

```css
:root {
    --primary-color: #your-color;
}
```

### Add Real API

Replace `fetchMetrics` function:

```javascript
async function fetchMetrics(dateRange) {
    const response = await fetch('/api/metrics', {
        method: 'POST',
        body: JSON.stringify({ dateRange })
    });
    return await response.json();
}
```

## 📚 Documentation

- **Deployment**: See `DEPLOYMENT_GUIDE.md` for complete Vercel setup
- **Running**: See `HOW_TO_RUN.md` for all running options
- **Architecture**: See `ANALYTICS_README.md` for design details
- **Main Docs**: See `../IMPLEMENTATION_SUMMARY.md` for feature overview

## 🔧 Development

```bash
# Install dependencies (optional)
cd examples
npm install

# Run locally
npm run dev

# Deploy preview
npm run deploy:preview

# Deploy production
npm run deploy
```

## 🌐 Deployment Options

### Vercel (Recommended)
```bash
vercel --prod
```
**Pros**: Instant, free tier, global CDN, automatic HTTPS

### Netlify
```bash
netlify deploy --prod
```
**Pros**: Similar to Vercel, good free tier

### GitHub Pages
```bash
# Push to gh-pages branch
git subtree push --prefix examples origin gh-pages
```
**Pros**: Free for public repos

### AWS S3 + CloudFront
```bash
aws s3 sync . s3://your-bucket --exclude "*.md"
```
**Pros**: Highly scalable, full control

## 📈 Monitoring

After deployment, monitor:
- **Vercel Analytics**: Built-in performance tracking
- **Google Analytics**: Add tracking code to HTML
- **Sentry**: Error tracking and monitoring

## 🤝 Contributing

Improvements welcome! To contribute:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test locally: `npm run dev`
5. Submit a pull request

## 📝 License

MIT License - see main repository for details

## 🆘 Support

- **Issues**: https://github.com/ravensone/ravensone/issues
- **Discussions**: https://github.com/ravensone/ravensone/discussions
- **Vercel Docs**: https://vercel.com/docs

## 🎓 Learn More

- **RavensOne Language**: See `../README.md`
- **Type System**: See `../src/types.rs`
- **Reactive System**: See `../src/reactive.rs`
- **SSR Engine**: See `../src/ssr.rs`

---

## 🚀 One-Line Deploy

```bash
git clone https://github.com/ravensone/ravensone.git && cd ravensone/examples && vercel --prod
```

That's it! Your analytics dashboard is live. 🎉

---

**Built with RavensOne** | [Documentation](../IMPLEMENTATION_SUMMARY.md) | [GitHub](https://github.com/ravensone/ravensone)
