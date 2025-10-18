# RavensOne Registry - Fly.io Deployment Guide

Complete guide to deploying the RavensOne package registry to Fly.io.

---

## Prerequisites

1. **Fly.io Account** - Sign up at https://fly.io/app/sign-up
2. **Fly CLI** - Install the CLI tool
3. **Docker** - For local testing (optional)

---

## Step 1: Install Fly CLI

```bash
# macOS/Linux
curl -L https://fly.io/install.sh | sh

# Or via Homebrew (macOS)
brew install flyctl

# Verify installation
flyctl version
```

---

## Step 2: Login to Fly.io

```bash
flyctl auth login
```

This will open your browser for authentication.

---

## Step 3: Create PostgreSQL Database

```bash
# Navigate to registry directory
cd registry

# Create a PostgreSQL cluster (free tier: shared-cpu, 256MB)
flyctl postgres create \
  --name ravensone-registry-db \
  --region sjc \
  --initial-cluster-size 1 \
  --vm-size shared-cpu-1x \
  --volume-size 1

# Save the connection string shown in the output!
# It will look like: postgres://postgres:password@top2.nearest.of.ravensone-registry-db.internal:5432
```

**Note:** The database password will only be shown once! Save it securely.

---

## Step 4: Create Fly.io App

```bash
# Create the app (don't deploy yet)
flyctl apps create ravensone-registry --org personal

# Or if you want to choose a different name:
# flyctl apps create your-custom-name --org personal
```

---

## Step 5: Attach Database to App

```bash
# Attach the PostgreSQL database
flyctl postgres attach ravensone-registry-db --app ravensone-registry

# This creates a DATABASE_URL secret automatically
```

---

## Step 6: Set Environment Secrets

```bash
# Generate a secure JWT secret (32+ characters)
# You can use this command to generate one:
openssl rand -base64 32

# Set the JWT secret
flyctl secrets set JWT_SECRET="your-generated-secret-here" --app ravensone-registry

# Verify secrets
flyctl secrets list --app ravensone-registry
```

You should see:
- `DATABASE_URL` (from postgres attach)
- `JWT_SECRET` (just set)

---

## Step 7: Create Persistent Volume for Package Storage

```bash
# Create a 1GB volume for storing package tarballs
flyctl volumes create registry_storage \
  --region sjc \
  --size 1 \
  --app ravensone-registry
```

---

## Step 8: Deploy!

```bash
# Deploy the registry
flyctl deploy --app ravensone-registry

# Monitor deployment logs
flyctl logs --app ravensone-registry
```

**First deployment takes ~5-10 minutes** (building Rust from scratch).

---

## Step 9: Run Database Migrations

After first deployment, run the schema migrations:

```bash
# SSH into the app
flyctl ssh console --app ravensone-registry

# Inside the container, run psql
psql $DATABASE_URL

# Copy and paste the SQL from registry/migrations/20251017_init.sql
# (The full schema with CREATE TABLE statements)

# Exit psql
\q

# Exit SSH
exit
```

**Alternatively**, you can run migrations from your local machine:

```bash
# Get the database connection string
flyctl postgres db-url ravensone-registry-db

# Use psql locally
psql "postgres://postgres:password@..." < migrations/20251017_init.sql
```

---

## Step 10: Test the Deployment

```bash
# Check app status
flyctl status --app ravensone-registry

# Get the app URL
flyctl info --app ravensone-registry

# Test health endpoint
curl https://ravensone-registry.fly.dev/health
# Should return: OK

# Test user registration
curl -X POST https://ravensone-registry.fly.dev/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"testpass123"}'
```

---

## Step 11: Update Client to Use Production URL

Edit `src/package_manager/registry.rs`:

```rust
pub fn new() -> Self {
    let home = dirs::home_dir().expect("Could not find home directory");
    let raven_dir = home.join(".raven");

    // Use production registry by default
    let base_url = std::env::var("RAVEN_REGISTRY")
        .unwrap_or_else(|_| "https://ravensone-registry.fly.dev/api/v1".to_string());

    RegistryClient {
        base_url,
        token: None,
        credentials_path: raven_dir.join("credentials.json"),
    }
}
```

Then rebuild:

```bash
cd ..  # Back to project root
cargo build --release
```

---

## Useful Commands

### View Logs
```bash
flyctl logs --app ravensone-registry
```

### SSH into Container
```bash
flyctl ssh console --app ravensone-registry
```

### Scale Resources (if needed)
```bash
# Increase memory to 512MB
flyctl scale memory 512 --app ravensone-registry

# Increase CPU
flyctl scale vm shared-cpu-2x --app ravensone-registry
```

### Redeploy After Changes
```bash
flyctl deploy --app ravensone-registry
```

### View Database Connection String
```bash
flyctl postgres db-url ravensone-registry-db
```

### Destroy Everything (if needed)
```bash
# Delete the app
flyctl apps destroy ravensone-registry

# Delete the database
flyctl postgres destroy ravensone-registry-db
```

---

## Environment Variables Reference

| Variable | Value | Set By |
|----------|-------|--------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres attach` |
| `JWT_SECRET` | 32+ character secret | `secrets set` |
| `PORT` | `8080` | `fly.toml` |
| `RUST_LOG` | `ravensone_registry=info` | `fly.toml` |
| `STORAGE_TYPE` | `local` | `fly.toml` |
| `STORAGE_PATH` | `/app/storage` | `fly.toml` |

---

## Troubleshooting

### App Won't Start

Check logs:
```bash
flyctl logs --app ravensone-registry
```

Common issues:
- **Database connection failed**: Check `DATABASE_URL` secret
- **Port binding**: Ensure app listens on `0.0.0.0:8080`
- **Missing migrations**: Run the SQL schema

### Health Check Failing

```bash
# Check if health endpoint works
flyctl ssh console --app ravensone-registry
curl localhost:8080/health
```

### Database Connection Issues

```bash
# Test database connection
flyctl postgres connect -a ravensone-registry-db
```

### Out of Memory

```bash
# Increase memory
flyctl scale memory 512 --app ravensone-registry
```

---

## Cost Estimate

**Free Tier (Hobby Plan):**
- PostgreSQL: Free (256MB, shared-cpu)
- Web App: Free (256MB RAM, shared-cpu, auto-sleep)
- Volume: Free (1GB)
- **Total**: $0/month

**If you exceed free tier:**
- PostgreSQL: ~$1.94/month (shared-cpu-1x, 1GB storage)
- Web App: ~$1.94/month (shared-cpu-1x, 256MB)
- Volume: Free (up to 3GB)
- **Total**: ~$4/month

---

## Production Readiness Checklist

- [ ] Database migrations run successfully
- [ ] JWT_SECRET set to strong random value
- [ ] Health endpoint responding
- [ ] User registration working
- [ ] Package publishing working
- [ ] Package search working
- [ ] Package download working
- [ ] HTTPS working (automatic via Fly.io)
- [ ] Monitoring set up (Fly.io dashboard)
- [ ] Backups enabled for database

---

## Next Steps After Deployment

1. **Update client** to use production URL
2. **Re-publish seed packages** to production registry
3. **Test full workflow** (register, login, publish, search, install)
4. **Set up custom domain** (optional): `ravensone.dev`
5. **Enable database backups**
6. **Set up alerts** for downtime

---

## Custom Domain Setup (Optional)

```bash
# Add custom domain
flyctl certs create registry.ravensone.dev --app ravensone-registry

# Get DNS records to configure
flyctl certs show registry.ravensone.dev --app ravensone-registry

# Add the CNAME record to your DNS provider
```

---

## Support

- **Fly.io Docs**: https://fly.io/docs/
- **Fly.io Community**: https://community.fly.io/
- **RavensOne Issues**: https://github.com/jrezin1201/RavensOne/issues

---

**Ready to deploy!** 🚀

Run these commands in order:
```bash
cd registry
flyctl auth login
flyctl postgres create --name ravensone-registry-db --region sjc
flyctl apps create ravensone-registry
flyctl postgres attach ravensone-registry-db
flyctl secrets set JWT_SECRET="$(openssl rand -base64 32)"
flyctl volumes create registry_storage --region sjc --size 1
flyctl deploy
```
