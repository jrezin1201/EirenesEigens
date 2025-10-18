#!/bin/bash
# RavensOne Registry - Fly.io Deployment Script
# This script automates the deployment of the RavensOne package registry

set -e  # Exit on any error

echo "🚀 RavensOne Registry - Fly.io Deployment"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Check if logged in
echo -e "${BLUE}Step 1: Checking Fly.io authentication...${NC}"
if ! flyctl auth whoami &> /dev/null; then
    echo -e "${YELLOW}⚠️  Not logged in to Fly.io${NC}"
    echo "Please run: flyctl auth login"
    echo "Then run this script again."
    exit 1
fi
echo -e "${GREEN}✅ Authenticated with Fly.io${NC}"
echo ""

# Step 2: Create PostgreSQL database
echo -e "${BLUE}Step 2: Creating PostgreSQL database...${NC}"
if flyctl postgres list | grep -q "ravensone-registry-db"; then
    echo -e "${YELLOW}⚠️  Database 'ravensone-registry-db' already exists${NC}"
else
    echo "Creating PostgreSQL cluster..."
    flyctl postgres create \
        --name ravensone-registry-db \
        --region sjc \
        --initial-cluster-size 1 \
        --vm-size shared-cpu-1x \
        --volume-size 1
    echo -e "${GREEN}✅ Database created${NC}"
fi
echo ""

# Step 3: Create the app
echo -e "${BLUE}Step 3: Creating Fly.io app...${NC}"
if flyctl status --app ravensone-registry &> /dev/null; then
    echo -e "${YELLOW}⚠️  App 'ravensone-registry' already exists${NC}"
else
    echo "Creating app..."
    flyctl apps create ravensone-registry
    echo -e "${GREEN}✅ App created${NC}"
fi
echo ""

# Step 4: Attach database
echo -e "${BLUE}Step 4: Attaching database to app...${NC}"
if flyctl secrets list --app ravensone-registry 2>/dev/null | grep -q DATABASE_URL; then
    echo -e "${YELLOW}⚠️  Database already attached${NC}"
else
    flyctl postgres attach ravensone-registry-db --app ravensone-registry
    echo -e "${GREEN}✅ Database attached${NC}"
fi
echo ""

# Step 5: Generate and set JWT secret
echo -e "${BLUE}Step 5: Setting environment secrets...${NC}"
if flyctl secrets list --app ravensone-registry 2>/dev/null | grep -q JWT_SECRET; then
    echo -e "${YELLOW}⚠️  JWT_SECRET already configured${NC}"
else
    JWT_SECRET=$(openssl rand -base64 32)
    flyctl secrets set JWT_SECRET="$JWT_SECRET" --app ravensone-registry
    echo -e "${GREEN}✅ Secrets configured${NC}"
fi
echo ""

# Step 6: Create storage volume
echo -e "${BLUE}Step 6: Creating persistent storage volume...${NC}"
if flyctl volumes list --app ravensone-registry | grep -q "registry_storage"; then
    echo -e "${YELLOW}⚠️  Volume 'registry_storage' already exists${NC}"
else
    flyctl volumes create registry_storage \
        --region sjc \
        --size 1 \
        --yes \
        --app ravensone-registry
    echo -e "${GREEN}✅ Volume created${NC}"
fi
echo ""

# Step 7: Deploy the app
echo -e "${BLUE}Step 7: Deploying registry server...${NC}"
echo "This may take 5-10 minutes (building Rust from scratch)..."
flyctl deploy --app ravensone-registry
echo -e "${GREEN}✅ Deployment complete${NC}"
echo ""

# Step 8: Run migrations
echo -e "${BLUE}Step 8: Running database migrations...${NC}"
echo "Getting database connection string..."
DB_URL=$(flyctl postgres db-url ravensone-registry-db)

echo "Running SQL migrations..."
psql "$DB_URL" < migrations/20251017_init.sql

echo -e "${GREEN}✅ Migrations complete${NC}"
echo ""

# Step 9: Check status
echo -e "${BLUE}Step 9: Verifying deployment...${NC}"
flyctl status --app ravensone-registry
echo ""

# Step 10: Test health endpoint
echo -e "${BLUE}Step 10: Testing health endpoint...${NC}"
sleep 5  # Give the app time to start
HEALTH_CHECK=$(curl -s https://ravensone-registry.fly.dev/health)
if [ "$HEALTH_CHECK" = "OK" ]; then
    echo -e "${GREEN}✅ Health check passed!${NC}"
else
    echo -e "${YELLOW}⚠️  Health check returned: $HEALTH_CHECK${NC}"
fi
echo ""

# Final summary
echo "=========================================="
echo -e "${GREEN}🎉 Deployment Complete!${NC}"
echo ""
echo "Registry URL: https://ravensone-registry.fly.dev"
echo "API Endpoint: https://ravensone-registry.fly.dev/api/v1"
echo ""
echo "Next steps:"
echo "1. Test user registration:"
echo "   curl -X POST https://ravensone-registry.fly.dev/api/v1/auth/register \\"
echo "     -H \"Content-Type: application/json\" \\"
echo "     -d '{\"username\":\"testuser\",\"email\":\"test@example.com\",\"password\":\"testpass123\"}'"
echo ""
echo "2. Update client to use production URL:"
echo "   Edit src/package_manager/registry.rs"
echo "   Change base_url to: https://ravensone-registry.fly.dev/api/v1"
echo ""
echo "3. View logs:"
echo "   flyctl logs --app ravensone-registry"
echo ""
echo "4. SSH into container:"
echo "   flyctl ssh console --app ravensone-registry"
echo ""
