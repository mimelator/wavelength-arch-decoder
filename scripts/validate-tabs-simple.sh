#!/bin/bash

# Simple validation script using curl to test API endpoints
# This is faster and doesn't require browser automation

REPO_ID="${1:-f424b3dc-f3c0-440f-89f7-bf1219d693ec}"
BASE_URL="${BASE_URL:-http://localhost:8080}"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}============================================================${NC}"
echo -e "${BLUE}Repository Tab Validation Script (API-based)${NC}"
echo -e "${BLUE}============================================================${NC}"
echo -e "${CYAN}Repository ID: ${REPO_ID}${NC}"
echo -e "${CYAN}Base URL: ${BASE_URL}${NC}\n"

# Test endpoints
ENDPOINTS=(
    "dependencies:/api/v1/repositories/${REPO_ID}/dependencies"
    "services:/api/v1/repositories/${REPO_ID}/services"
    "code:/api/v1/repositories/${REPO_ID}/code/elements"
    "security:/api/v1/repositories/${REPO_ID}/security"
    "tools:/api/v1/repositories/${REPO_ID}/tools"
    "tests:/api/v1/repositories/${REPO_ID}/tests"
    "documentation:/api/v1/repositories/${REPO_ID}/documentation"
)

SUCCESS_COUNT=0
FAIL_COUNT=0
RESULTS=()

for endpoint_pair in "${ENDPOINTS[@]}"; do
    IFS=':' read -r name endpoint <<< "$endpoint_pair"
    full_url="${BASE_URL}${endpoint}"
    
    echo -e "${CYAN}Testing ${name}...${NC}"
    
    # Make API call
    response=$(curl -s -w "\n%{http_code}" "${full_url}" 2>&1)
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        # Try to parse JSON and count items
        if command -v jq &> /dev/null; then
            count=$(echo "$body" | jq 'length' 2>/dev/null || echo "?")
            if [ "$count" != "?" ] && [ "$count" != "null" ]; then
                echo -e "${GREEN}  ✓ ${name}: OK (${count} items)${NC}"
                RESULTS+=("${GREEN}✓${NC} ${name}: ${count} items")
                ((SUCCESS_COUNT++))
            else
                # Check if it's an object with a data array
                count=$(echo "$body" | jq '.data | length' 2>/dev/null || echo "?")
                if [ "$count" != "?" ] && [ "$count" != "null" ]; then
                    echo -e "${GREEN}  ✓ ${name}: OK (${count} items)${NC}"
                    RESULTS+=("${GREEN}✓${NC} ${name}: ${count} items")
                    ((SUCCESS_COUNT++))
                else
                    echo -e "${GREEN}  ✓ ${name}: OK (response received)${NC}"
                    RESULTS+=("${GREEN}✓${NC} ${name}: response received")
                    ((SUCCESS_COUNT++))
                fi
            fi
        else
            # No jq, just check if response is not empty
            if [ -n "$body" ] && [ "$body" != "null" ] && [ "$body" != "[]" ]; then
                echo -e "${GREEN}  ✓ ${name}: OK (response received)${NC}"
                RESULTS+=("${GREEN}✓${NC} ${name}: response received")
                ((SUCCESS_COUNT++))
            else
                echo -e "${YELLOW}  ⚠ ${name}: OK but empty response (may be expected)${NC}"
                RESULTS+=("${YELLOW}⚠${NC} ${name}: empty (may be expected)")
                ((SUCCESS_COUNT++))
            fi
        fi
    else
        echo -e "${RED}  ✗ ${name}: Failed (HTTP ${http_code})${NC}"
        if [ -n "$body" ]; then
            error_msg=$(echo "$body" | head -c 100)
            echo -e "${RED}    Error: ${error_msg}...${NC}"
        fi
        RESULTS+=("${RED}✗${NC} ${name}: HTTP ${http_code}")
        ((FAIL_COUNT++))
    fi
    echo ""
done

# Print summary
echo -e "${BLUE}============================================================${NC}"
echo -e "${BLUE}Validation Summary${NC}"
echo -e "${BLUE}============================================================${NC}\n"

for result in "${RESULTS[@]}"; do
    echo -e "  ${result}"
done

echo ""
echo -e "${BLUE}============================================================${NC}"
if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ All endpoints validated successfully!${NC}"
    echo -e "${GREEN}  (${SUCCESS_COUNT}/${#ENDPOINTS[@]} endpoints passed)${NC}"
    exit 0
else
    echo -e "${RED}⚠ Some endpoints failed validation${NC}"
    echo -e "${RED}  (${SUCCESS_COUNT}/${#ENDPOINTS[@]} passed, ${FAIL_COUNT} failed)${NC}"
    exit 1
fi

