# Deploying RavensOne Analytics Dashboard to Vercel

Complete step-by-step guide for deploying the production application to Vercel.

---

## Prerequisites

Before starting, ensure you have:

- ✅ A GitHub account
- ✅ A Vercel account (free tier works fine)
- ✅ Git installed on your computer
- ✅ Node.js installed (v18 or higher)

---

## Step 1: Clone the Repository

Open your terminal and clone the RavensOne repository:

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/ravensone.git

# Navigate to the directory
cd ravensone

# Verify the example files exist
ls examples/
```

You should see:
- `analytics_dashboard.raven`
- `analytics_dashboard.css`
- `run_dashboard.html`
- `ANALYTICS_README.md`
- `HOW_TO_RUN.md`
- `DEPLOYMENT_GUIDE.md` (this file)

---

## Step 2: Prepare for Deployment

### 2.1 Create Vercel Configuration

Create a `vercel.json` file in the `examples/` directory:

```bash
cd examples
touch vercel.json
```

Edit `vercel.json` with the following content:

```json
{
  "version": 2,
  "name": "ravensone-analytics-dashboard",
  "builds": [
    {
      "src": "run_dashboard.html",
      "use": "@vercel/static"
    },
    {
      "src": "analytics_dashboard.css",
      "use": "@vercel/static"
    }
  ],
  "routes": [
    {
      "src": "/",
      "dest": "/run_dashboard.html"
    },
    {
      "src": "/analytics_dashboard.css",
      "dest": "/analytics_dashboard.css"
    }
  ]
}
```

### 2.2 Create Package.json (Optional but Recommended)

Create a `package.json` file in the `examples/` directory:

```bash
touch package.json
```

Edit `package.json`:

```json
{
  "name": "ravensone-analytics-dashboard",
  "version": "1.0.0",
  "description": "Production analytics dashboard built with RavensOne",
  "private": true,
  "scripts": {
    "dev": "open run_dashboard.html",
    "build": "echo 'Static site - no build needed'",
    "start": "echo 'Deployed to Vercel'"
  },
  "keywords": [
    "ravensone",
    "analytics",
    "dashboard",
    "reactive",
    "webassembly"
  ],
  "author": "Your Name",
  "license": "MIT"
}
```

---

## Step 3: Test Locally

Before deploying, test the application locally:

```bash
# Option 1: Open directly in browser
open run_dashboard.html

# Option 2: Use Python's built-in server
python3 -m http.server 8000
# Then visit: http://localhost:8000/run_dashboard.html

# Option 3: Use Node.js http-server (install if needed)
npx http-server -p 8000
# Then visit: http://localhost:8000/run_dashboard.html
```

Verify that:
- ✅ Dashboard loads correctly
- ✅ Metrics are displayed
- ✅ Date range selector works
- ✅ Charts render properly
- ✅ No console errors

---

## Step 4: Push to GitHub

If you haven't already, push your code to GitHub:

```bash
# Navigate back to the root directory
cd ..

# Initialize git (if not already initialized)
git init

# Add all files
git add .

# Commit
git commit -m "Add RavensOne Analytics Dashboard for deployment"

# Add your GitHub repository as remote
git remote add origin https://github.com/YOUR_USERNAME/ravensone.git

# Push to GitHub
git push -u origin main
```

---

## Step 5: Deploy to Vercel

### Method A: Deploy via Vercel CLI (Recommended)

#### Install Vercel CLI

```bash
npm install -g vercel
```

#### Login to Vercel

```bash
vercel login
```

Follow the prompts to authenticate with your Vercel account.

#### Deploy

```bash
# Navigate to examples directory
cd examples

# Deploy to Vercel
vercel
```

You'll be prompted with several questions:

1. **Set up and deploy "~/ravensone/examples"?** → Press `Y`
2. **Which scope do you want to deploy to?** → Select your account
3. **Link to existing project?** → Press `N` (for first deployment)
4. **What's your project's name?** → `ravensone-analytics-dashboard`
5. **In which directory is your code located?** → Press Enter (current directory)
6. **Want to override the settings?** → Press `N`

Vercel will deploy your application and provide a URL like:
```
https://ravensone-analytics-dashboard.vercel.app
```

#### Deploy to Production

```bash
# Deploy to production
vercel --prod
```

---

### Method B: Deploy via Vercel Dashboard

#### 1. Go to Vercel Dashboard

Visit [vercel.com](https://vercel.com) and log in.

#### 2. Import Project

- Click **"Add New..."** → **"Project"**
- Click **"Import Git Repository"**
- Select your GitHub repository (`ravensone`)
- Click **"Import"**

#### 3. Configure Project

**Root Directory:**
- Set to `examples`
- Click **"Edit"** next to Root Directory
- Enter: `examples`

**Build & Development Settings:**
- Framework Preset: `Other`
- Build Command: Leave empty (static site)
- Output Directory: Leave empty
- Install Command: Leave empty

**Environment Variables:**
- No environment variables needed for this demo

#### 4. Deploy

- Click **"Deploy"**
- Wait 30-60 seconds for deployment to complete
- You'll get a URL like: `https://ravensone-analytics-dashboard.vercel.app`

---

## Step 6: Configure Custom Domain (Optional)

### Via Vercel Dashboard

1. Go to your project in Vercel Dashboard
2. Click **"Settings"** → **"Domains"**
3. Click **"Add Domain"**
4. Enter your domain (e.g., `analytics.yourdomain.com`)
5. Follow the DNS configuration instructions
6. Wait for DNS propagation (usually 5-30 minutes)

### Via Vercel CLI

```bash
vercel domains add analytics.yourdomain.com
```

---

## Step 7: Verify Deployment

Visit your deployed URL and verify:

- ✅ Dashboard loads within 2 seconds
- ✅ All 4 metrics are visible
- ✅ Charts render correctly
- ✅ Date range selector functions
- ✅ Loading states work
- ✅ No console errors
- ✅ Mobile responsive design works

### Performance Testing

Use [PageSpeed Insights](https://pagespeed.web.dev/):
```
https://pagespeed.web.dev/analysis?url=YOUR_VERCEL_URL
```

Expected scores:
- Performance: 95-100
- Accessibility: 90-100
- Best Practices: 95-100
- SEO: 90-100

---

## Step 8: Set Up Continuous Deployment

Vercel automatically sets up continuous deployment from GitHub.

### How it Works:

1. **Push to GitHub** → Automatically triggers deployment
2. **Pull Requests** → Get preview deployments
3. **Main Branch** → Deploys to production

### To Deploy Updates:

```bash
# Make changes to your code
# Edit examples/run_dashboard.html or analytics_dashboard.css

# Commit changes
git add .
git commit -m "Update dashboard metrics"

# Push to GitHub
git push origin main
```

Vercel will automatically:
- Detect the push
- Build and deploy
- Update your live site
- Send you a notification

---

## Advanced Configuration

### Environment Variables (For Future API Integration)

When you add real backend APIs, configure environment variables:

#### Via Vercel Dashboard:
1. Project Settings → Environment Variables
2. Add variables:
   - `API_URL`: Your backend API URL
   - `API_KEY`: Your API key (mark as sensitive)

#### Via Vercel CLI:
```bash
vercel env add API_URL
vercel env add API_KEY
```

### Custom Headers (Security)

Create `vercel.json` with headers:

```json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        {
          "key": "X-Content-Type-Options",
          "value": "nosniff"
        },
        {
          "key": "X-Frame-Options",
          "value": "DENY"
        },
        {
          "key": "X-XSS-Protection",
          "value": "1; mode=block"
        },
        {
          "key": "Referrer-Policy",
          "value": "strict-origin-when-cross-origin"
        }
      ]
    }
  ]
}
```

### Analytics Integration

Add Vercel Analytics to track performance:

```bash
# Install Vercel Analytics
npm install @vercel/analytics
```

Update `run_dashboard.html` before `</body>`:

```html
<script>
  window.va = window.va || function () { (window.vaq = window.vaq || []).push(arguments); };
</script>
<script defer src="/_vercel/insights/script.js"></script>
```

---

## Troubleshooting

### Issue: 404 Not Found

**Solution:**
- Verify `vercel.json` routes are correct
- Ensure `run_dashboard.html` exists in examples directory
- Check Root Directory setting in Vercel

### Issue: CSS Not Loading

**Solution:**
- Verify `analytics_dashboard.css` is in the same directory
- Check browser console for CORS errors
- Ensure CSS path in HTML is correct: `<link rel="stylesheet" href="analytics_dashboard.css">`

### Issue: Slow Load Times

**Solution:**
- Enable Vercel Edge Network (automatic)
- Minimize JavaScript payload
- Optimize images (if any)
- Use Vercel's built-in CDN

### Issue: Charts Not Rendering

**Solution:**
- Check JavaScript console for errors
- Verify Canvas API is supported (all modern browsers)
- Ensure JavaScript is enabled
- Try in different browser

### Issue: Deployment Fails

**Solution:**
```bash
# Check Vercel logs
vercel logs

# Redeploy
vercel --prod --force

# Clear Vercel cache
vercel --prod --no-cache
```

---

## Production Checklist

Before going live, verify:

- [ ] All links work (no broken links)
- [ ] Forms work (date selector)
- [ ] Charts render on all devices
- [ ] Mobile responsive (test on phone)
- [ ] Page loads < 3 seconds
- [ ] No console errors
- [ ] Favicon added (optional)
- [ ] Meta tags for SEO (in HTML head)
- [ ] HTTPS enabled (automatic with Vercel)
- [ ] Analytics tracking configured
- [ ] Error tracking set up (optional)
- [ ] Monitoring configured (optional)

---

## Performance Optimization

### Enable Compression

Vercel automatically enables gzip/brotli compression.

Verify:
```bash
curl -H "Accept-Encoding: gzip" -I https://your-app.vercel.app
```

### Add Caching Headers

Update `vercel.json`:

```json
{
  "headers": [
    {
      "source": "/(.*).css",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    }
  ]
}
```

### Enable Edge Caching

All static assets are automatically cached at Vercel's edge network (60+ locations worldwide).

---

## Monitoring & Analytics

### Vercel Analytics

View in Vercel Dashboard:
- Page views
- Unique visitors
- Performance metrics
- Geographic distribution

### Custom Analytics

Add Google Analytics:

```html
<!-- Add to <head> in run_dashboard.html -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'GA_MEASUREMENT_ID');
</script>
```

### Error Tracking

Add Sentry:

```html
<script src="https://browser.sentry-cdn.com/7.x.x/bundle.min.js"></script>
<script>
  Sentry.init({ dsn: 'YOUR_SENTRY_DSN' });
</script>
```

---

## Cost Estimation

**Vercel Free Tier** (more than enough for demo):
- ✅ 100 GB bandwidth/month
- ✅ Unlimited deployments
- ✅ Automatic HTTPS
- ✅ Global CDN
- ✅ Preview deployments
- ✅ Analytics included

**Pro Tier** ($20/month - if needed):
- 1 TB bandwidth
- Advanced analytics
- Password protection
- Custom deployment regions

For the demo dashboard: **Free tier is perfect!**

---

## Next Steps

After deploying:

1. **Share Your App**
   - Tweet the URL with #RavensOne
   - Add to your portfolio
   - Share on LinkedIn

2. **Monitor Performance**
   - Check Vercel Analytics weekly
   - Review error logs
   - Monitor load times

3. **Iterate & Improve**
   - Add more metrics
   - Customize styling
   - Integrate real APIs
   - Add user authentication

4. **Scale When Ready**
   - Upgrade to Pro tier if needed
   - Add custom domain
   - Implement CDN optimizations

---

## Support

Need help?

- **Vercel Docs**: https://vercel.com/docs
- **Vercel Community**: https://github.com/vercel/vercel/discussions
- **RavensOne Issues**: https://github.com/ravensone/ravensone/issues
- **Email Support**: support@vercel.com (for Vercel issues)

---

## Success! 🎉

Your RavensOne Analytics Dashboard is now live on Vercel!

**What you've accomplished:**
- ✅ Deployed a production-ready application
- ✅ Set up continuous deployment
- ✅ Configured global CDN
- ✅ Enabled HTTPS
- ✅ Production-grade performance

**Share your deployment:**
```
🚀 Just deployed RavensOne Analytics Dashboard to Vercel!

Check it out: [YOUR_VERCEL_URL]

Built with:
- Type-safe reactive state
- Server-side rendering
- Progressive hydration
- WebAssembly performance

#RavensOne #WebAssembly #Vercel #WebDev
```

---

**Deployment Date**: 2025-10-17
**Guide Version**: 1.0
**Last Updated**: 2025-10-17

Happy deploying! 🚀
