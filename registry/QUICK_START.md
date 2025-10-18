# RavensOne Registry - Quick Start Deployment

## 🚀 Deploy in 3 Steps

### Step 1: Login to Fly.io

```bash
flyctl auth login
```

This will open your browser for authentication.

### Step 2: Run the Deployment Script

```bash
cd registry
./deploy.sh
```

The script will automatically:
- ✅ Create PostgreSQL database
- ✅ Create Fly.io app
- ✅ Attach database
- ✅ Set environment secrets
- ✅ Create storage volume
- ✅ Deploy the registry
- ✅ Run migrations
- ✅ Test the deployment

**Deployment takes ~5-10 minutes** (first time builds Rust from scratch)

### Step 3: Update Client to Use Production

After deployment, update the client:

**File:** `src/package_manager/registry.rs`

**Change:**
```rust
let base_url = std::env::var("RAVEN_REGISTRY")
    .unwrap_or_else(|_| "https://ravensone-registry.fly.dev/api/v1".to_string());
```

Then rebuild:
```bash
cargo build --release
```

---

## 🧪 Test the Deployment

### Test 1: Health Check
```bash
curl https://ravensone-registry.fly.dev/health
# Should return: OK
```

### Test 2: Register a User
```bash
curl -X POST https://ravensone-registry.fly.dev/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"testpass123"}'
```

### Test 3: Search Packages
```bash
curl https://ravensone-registry.fly.dev/api/v1/packages/search?q=raven
```

---

## 📊 Monitoring

### View Logs
```bash
flyctl logs --app ravensone-registry
```

### Check Status
```bash
flyctl status --app ravensone-registry
```

### SSH into Container
```bash
flyctl ssh console --app ravensone-registry
```

### Database Console
```bash
flyctl postgres connect -a ravensone-registry-db
```

---

## 🔧 Management Commands

### Redeploy After Changes
```bash
flyctl deploy --app ravensone-registry
```

### Scale Resources
```bash
# Increase memory
flyctl scale memory 512 --app ravensone-registry

# Increase CPU
flyctl scale vm shared-cpu-2x --app ravensone-registry
```

### Restart App
```bash
flyctl apps restart ravensone-registry
```

---

## 🌐 Production URLs

- **Registry**: https://ravensone-registry.fly.dev
- **API**: https://ravensone-registry.fly.dev/api/v1
- **Health**: https://ravensone-registry.fly.dev/health

---

## 💰 Cost

**Free Tier:**
- PostgreSQL: Free (256MB, shared-cpu)
- Web App: Free (256MB RAM, shared-cpu, auto-sleep)
- Volume: Free (1GB)
- **Total: $0/month**

**If you exceed free tier:**
- ~$4/month total

---

## 🐛 Troubleshooting

### App Won't Start
```bash
# Check logs
flyctl logs --app ravensone-registry

# Common issues:
# - Database connection failed → Check DATABASE_URL secret
# - Port binding → Ensure app listens on 0.0.0.0:8080
# - Missing migrations → Run SQL schema
```

### Database Issues
```bash
# Test connection
flyctl postgres connect -a ravensone-registry-db

# Check tables
\dt
```

### Out of Memory
```bash
# Increase memory
flyctl scale memory 512 --app ravensone-registry
```

---

## 🎉 You're Live!

Once deployed, you can:
1. Use the registry with `raven pkg` commands
2. Publish your packages
3. Share packages with others
4. Build the RavensOne ecosystem!

---

**Questions?** Check the full deployment guide in `DEPLOYMENT.md`
