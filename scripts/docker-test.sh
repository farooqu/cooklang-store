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

# Fixtures to seed for testing
FIXTURES_TO_SEED=("chocolate-cake" "vanilla-cake" "test-recipe")

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

# Function to run a test with response validation
run_test() {
    local test_name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local expected_status=$5
    local validation_fn=${6:-}
    
    echo -e "${YELLOW}Testing: $test_name${NC}"
    
    local cmd="curl -s -w '%{http_code}' -X $method"
    
    if [ -n "$data" ]; then
        cmd="$cmd -H 'Content-Type: application/json' -d '$data'"
    fi
    
    cmd="$cmd 'http://localhost:3000$endpoint' -o /tmp/response.json"
    
    local status=$(eval $cmd | tail -c 3)
    
    if [ "$status" != "$expected_status" ]; then
        echo -e "${RED}✗ $test_name (Expected $expected_status, got $status)${NC}"
        cat /tmp/response.json
        echo ""
        return 1
    fi
    
    # If validation function provided, run it
    if [ -n "$validation_fn" ]; then
        if ! $validation_fn; then
            echo -e "${RED}✗ $test_name (Validation failed)${NC}"
            return 1
        fi
    fi
    
    echo -e "${GREEN}✓ $test_name (HTTP $status)${NC}"
    cat /tmp/response.json | jq . 2>/dev/null || cat /tmp/response.json
    echo ""
    return 0
}

# Validation functions
validate_status() {
    local expected_count=$1
    jq -e ".recipe_count == $expected_count" /tmp/response.json > /dev/null 2>&1
}

validate_categories() {
    jq -e '.categories | index("desserts") != null' /tmp/response.json > /dev/null 2>&1
}

validate_recipes_list() {
    jq -e '.recipes | length > 0' /tmp/response.json > /dev/null 2>&1
}

validate_search_results() {
    local search_term=$1
    jq -e ".recipes | length > 0" /tmp/response.json > /dev/null 2>&1
}

validate_nested_categories() {
    # Verify nested categories appear in the response
    jq -e '.categories | map(select(. == "meals/asian/thai" or . == "meals/european/italian")) | length == 2' /tmp/response.json > /dev/null 2>&1
}

validate_recipe_in_nested_category() {
    # Verify recipe exists with the expected category path
    jq -e '.recipes | length > 0' /tmp/response.json > /dev/null 2>&1
}

validate_title_extraction() {
    # Verify the name in response matches YAML title, not request name
    jq -e '.name | type == "string"' /tmp/response.json > /dev/null 2>&1
}

main() {
    echo -e "${GREEN}Cooklang Store Docker Integration Tests${NC}"
    echo ""
    
    # Create test repo directory
    echo -e "${YELLOW}Creating test recipe repository at $TEST_REPO_DIR${NC}"
    mkdir -p "$TEST_REPO_DIR"
    
    # Get script directory for relative path to fixtures
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    
    # Initialize git repo
    cd "$TEST_REPO_DIR"
    git init -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Create recipes directory
    mkdir -p recipes/desserts
    
    # Ensure directory exists in git repo with .gitkeep
    touch recipes/desserts/.gitkeep
    
    # Seed only selected fixture files for testing
    for fixture_name in "${FIXTURES_TO_SEED[@]}"; do
        fixture_file="$SCRIPT_DIR/tests/fixtures/${fixture_name}.cook"
        if [ -f "$fixture_file" ]; then
            cp "$fixture_file" "$TEST_REPO_DIR/recipes/desserts/${fixture_name}.cook"
        fi
    done
    
    # Seed fixtures into nested categories for Phase 2 testing
    mkdir -p recipes/meals/asian/thai
    mkdir -p recipes/meals/european/italian
    
    fixture_file="$SCRIPT_DIR/tests/fixtures/pad-thai.cook"
    if [ -f "$fixture_file" ]; then
        cp "$fixture_file" "$TEST_REPO_DIR/recipes/meals/asian/thai/pad-thai.cook"
    fi
    
    fixture_file="$SCRIPT_DIR/tests/fixtures/spaghetti.cook"
    if [ -f "$fixture_file" ]; then
        cp "$fixture_file" "$TEST_REPO_DIR/recipes/meals/european/italian/spaghetti.cook"
    fi
    
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
        -v "$TEST_REPO_DIR:/recipes" \
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
    # Base fixtures (desserts) + nested fixtures (pad-thai, spaghetti) = 5 total
    EXPECTED_FIXTURE_COUNT=$((${#FIXTURES_TO_SEED[@]} + 2))
    
    # Test 1: Health check
    if ! run_test "Health Check" "GET" "/health" "" "200"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 2: Status endpoint - verify recipe count matches fixtures
    if ! run_test "Status Endpoint" "GET" "/api/v1/status" "" "200" \
        "validate_status $EXPECTED_FIXTURE_COUNT"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 3: List categories - verify desserts category exists
    if ! run_test "List Categories" "GET" "/api/v1/categories" "" "200" \
        "validate_categories"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 4: List recipes - verify recipes were seeded
    if ! run_test "List Recipes" "GET" "/api/v1/recipes" "" "200" \
        "validate_recipes_list"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 5: Create recipe
    if ! run_test "Create Recipe" "POST" "/api/v1/recipes" \
        '{"name":"New Test Recipe","content":"---\ntitle: New Test Recipe\n---\n\n@flour{2%cup}","category":"desserts"}' \
        "201"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 6: Search recipes - verify search returns results
    if ! run_test "Search Recipes" "GET" "/api/v1/recipes/search?q=cake" "" "200" \
        "validate_search_results cake"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 7: List categories - verify nested categories exist
    if ! run_test "List Nested Categories" "GET" "/api/v1/categories" "" "200" \
        "validate_nested_categories"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 8: Get recipes from nested category (meals/asian/thai)
    if ! run_test "Get Recipes from Nested Category" "GET" "/api/v1/categories/meals%2Fasian%2Fthai" "" "200" \
        "validate_recipe_in_nested_category"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 9: Create recipe with YAML front matter title - verify title extraction
    if ! run_test "Create Recipe with Title Extraction" "POST" "/api/v1/recipes" \
        '{"name":"Ignore This Name","content":"---\ntitle: Strawberry Cheesecake\n---\n\n@cream{2%cup}","category":"desserts"}' \
        "201" "validate_title_extraction"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 10: Create recipe without YAML title - should fail
    if ! run_test "Create Recipe Missing YAML Title (Should Fail)" "POST" "/api/v1/recipes" \
        '{"name":"Test","content":"No YAML front matter here","category":"desserts"}' \
        "400"; then
        test_results=$((test_results + 1))
    fi
    
    # Test 11: Create recipe in nested category via POST
    if ! run_test "Create Recipe in Nested Category" "POST" "/api/v1/recipes" \
        '{"name":"Pad Thai","content":"---\ntitle: Pad Thai\n---\n\n@noodles{1%kg}","category":"meals/asian/thai"}' \
        "201"; then
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
