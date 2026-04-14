#!/bin/bash

# Script untuk testing FCM notification
# Usage: ./test_fcm_notification.sh <leave_id> <stage> <status>

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
BASE_URL="http://localhost:8080"
LEAVE_ID="${1:-leaves:xxxxx}"
STAGE="${2:-2}"
STATUS="${3:-APPROVED}"

echo -e "${YELLOW}=== FCM Notification Test ===${NC}"
echo ""
echo "Testing notification for:"
echo "  Leave ID: $LEAVE_ID"
echo "  Stage: $STAGE"
echo "  Status: $STATUS"
echo ""

# Test 1: Update leave status (this should trigger notification)
echo -e "${YELLOW}[1] Updating leave status...${NC}"
RESPONSE=$(curl -s -X POST "$BASE_URL/api/leaves/update-status" \
  -H "Content-Type: application/json" \
  -d "{
    \"id\": \"$LEAVE_ID\",
    \"stage\": $STAGE,
    \"status\": \"$STATUS\"
  }")

echo "Response: $RESPONSE"

if echo "$RESPONSE" | grep -q "success"; then
    echo -e "${GREEN}✓ Leave status updated successfully${NC}"
    echo -e "${GREEN}✓ Notification should be sent to user${NC}"
else
    echo -e "${RED}✗ Failed to update leave status${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}[2] Check backend logs for FCM notification status${NC}"
echo "Look for:"
echo "  - [INFO] Successfully sent FCM notification to <token>"
echo "  - [ERROR] Failed to send FCM notification: <error>"
echo ""

echo -e "${YELLOW}[3] Check Android app${NC}"
echo "  1. Open Android Studio Logcat"
echo "  2. Filter by 'FCM_SERVICE'"
echo "  3. Look for '=== FCM MESSAGE RECEIVED ==='"
echo "  4. Check notification tray on device/emulator"
echo ""

echo -e "${GREEN}Test completed!${NC}"
