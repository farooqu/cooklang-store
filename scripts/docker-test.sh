#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
TEST_REPO_DIR="/tmp/cooklang-test-recipes-$$"
CONTAINER_NAME="cooklang-store-test-$$"
TIMEOUT=30
API_BASE="http://localhost:3000/api/v1"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    
    # Stop and remove container
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
    
    # Remove test repo
    rm -rf "$TEST_REPO_DIR"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Function to wait for API to be ready
wait_for_api() {
    local elapsed=0
    echo -e "${YELLOW}Waiting for API to be ready...${NC}"
    
    while [ $elapsed -lt $TIMEOUT ]; do
        if curl -s http://localhost:3000/health >/dev/null 2>&1; then
            echo -e "${GREEN}API is ready${NC}"
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    
    echo -e "${RED}API failed to start within ${TIMEOUT}s${NC}"
    return 1
}

# Function to run a test
run_test() {
    local test_name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local expected_status=$5
    
    echo -e "${YELLOW}Testing: $test_name${NC}"
    
    local cmd="curl -s -w '%{http_code}' -X $method"
    
    if [ -n "$data" ]; then
        cmd="$cmd -H 'Content-Type: application/json' -d '$data'"
    fi
    
    cmd="$cmd 'http://localhost:3000$endpoint' -o /tmp/response.json"
    
    local status=$(eval $cmd | tail -c 3)
    
    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✓ $test_name (HTTP $status)${NC}"
        cat /tmp/response.json | jq . 2>/dev/null || cat /tmp/response.json
        echo ""
        return 0
    else
        echo -e "${RED}✗ $test_name (Expected $expected_status, got $status)${NC}"
        cat /tmp/response.json
        echo ""
        return 1
    fi
}

main() {
    echo -e "${GREEN}Cooklang Store Docker Integration Tests${NC}"
    echo ""
    
    # Create test repo directory
    echo -e "${YELLOW}Creating test recipe repository at $TEST_REPO_DIR${NC}"
    mkdir -p "$TEST_REPO_DIR"
    
    # Initialize git repo
    cd "$TEST_REPO_DIR"
    git init -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Create sample recipes directory and files
    mkdir -p recipes/desserts
    
    cat > recipes/desserts/chocolate-cake.cook << 'EOF'
Chocolate Cake

Ingredients:
- @flour{2%cup}
- @sugar{1%cup}
- @butter{0.5%cup}
- @eggs{2}
- @cocoa_powder{0.75%cup}
- @baking_powder{2%tsp}

Steps:
1. Preheat #oven{} to 350°F.
2. Cream @butter{} and @sugar{} together.
3. Beat in @eggs{} one at a time.
4. Mix @flour{}, @cocoa_powder{}, and @baking_powder{}.
5. Combine wet and dry ingredients.
6. Pour into #pan{} and bake for ~45{minutes}.
EOF
    
    git add .
    git commit -q -m "Initial test recipes"
    
    cd - > /dev/null
    
    # Build Docker image
    echo -e "${YELLOW}Building Docker image...${NC}"
    docker build -t cooklang-store-test . -q
    echo -e "${GREEN}Docker image built${NC}"
    echo ""
    
    # Run container
    echo -e "${YELLOW}Starting Docker container...${NC}"
    docker run -d \
        --name "$CONTAINER_NAME" \
        -p 3000:3000 \
        -v "$TEST_REPO_DIR:/data/recipes" \
        -e RUST_LOG=info \
        -e RECIPES_PATH=/data/recipes \
        cooklang-store-test
    
    echo -e "${GREEN}Container started${NC}"
    echo ""
    
    # Wait for API
    if ! wait_for_api; then
        echo -e "${RED}Failed to start API${NC}"
        docker logs "$CONTAINER_NAME"
        exit 1
    fi
    echo ""
    
    # Run tests
    test_results=0
    
    # Test 1: Health check
    if ! run_test "Health Check" "GET" "/health" "" "200"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 2: Status endpoint
    if ! run_test "Status Endpoint" "GET" "/api/v1/status" "" "200"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 3: List categories
    if ! run_test "List Categories" "GET" "/api/v1/categories" "" "200"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 4: List recipes
    if ! run_test "List Recipes" "GET" "/api/v1/recipes" "" "200"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 5: Create recipe
    if ! run_test "Create Recipe" "POST" "/api/v1/recipes" \
        '{"name":"Vanilla Cake","content":"Simple cake\n\n@flour{2%cup}","category":"desserts"}' \
        "201"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 6: Search recipes
    if ! run_test "Search Recipes" "GET" "/api/v1/recipes/search?q=cake" "" "200"; then
        test_results=$((test_results + 1))
    fi
    
    echo ""
    
    # Summary
    if [ $test_results -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}$test_results test(s) failed${NC}"
        exit 1
    fi
}

main "$@"
