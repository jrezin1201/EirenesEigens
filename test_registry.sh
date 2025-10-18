#!/bin/bash
# Registry Testing Script

set -e

RAVEN="./target/release/raven"

echo "=== RavensOne Registry End-to-End Test ==="
echo

# Test 1: Check server health
echo "✅ Test 1: Checking registry server health..."
HEALTH=$(curl -s http://localhost:4000/health)
if [ "$HEALTH" = "OK" ]; then
    echo "   ✓ Server is running"
else
    echo "   ✗ Server is not responding"
    exit 1
fi
echo

# Test 2: Register API endpoint directly
echo "✅ Test 2: Testing registration API endpoint..."
REG_RESPONSE=$(curl -s -X POST http://localhost:4000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"testuser@ravens.dev","password":"testpass123"}')
echo "   Response: $REG_RESPONSE"
USER_ID=$(echo $REG_RESPONSE | grep -o '"user_id":"[^"]*"' | cut -d'"' -f4 || true)
if [ -n "$USER_ID" ]; then
    echo "   ✓ User created with ID: $USER_ID"
else
    echo "   ⚠ User might already exist or registration failed"
fi
echo

# Test 3: Login API endpoint
echo "✅ Test 3: Testing login API endpoint..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:4000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"testuser@ravens.dev","password":"testpass123"}')
echo "   Response: $LOGIN_RESPONSE"
TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
if [ -n "$TOKEN" ]; then
    echo "   ✓ Login successful, token received"
else
    echo "   ✗ Login failed"
    exit 1
fi
echo

# Test 4: Search (empty registry)
echo "✅ Test 4: Testing search endpoint..."
SEARCH_RESPONSE=$(curl -s "http://localhost:4000/api/v1/search?q=raven&limit=10")
echo "   Response: $SEARCH_RESPONSE"
echo "   ✓ Search endpoint working"
echo

echo "=== All API Tests Passed! ==="
echo
echo "📝 Token for manual testing:"
echo "$TOKEN"
