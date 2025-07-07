#!/bin/bash

# Master test runner for all binding validation tests
# This script runs tests for all binding targets and compares results

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Nucleation Cross-Platform Binding Test Suite${NC}"
echo "=================================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Must be run from project root${NC}"
    exit 1
fi

# Function to print section headers
print_section() {
    echo ""
    echo -e "${BLUE}$1${NC}"
    echo "$(printf '%.0s-' {1..50})"
}

# Function to run command and check result
run_test() {
    local name="$1"
    local command="$2"
    
    echo -e "Running ${YELLOW}$name${NC}..."
    
    if eval "$command"; then
        echo -e "${GREEN}✅ $name passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $name failed${NC}"
        return 1
    fi
}

# Counters
total_tests=0
passed_tests=0

# Generate bindings first
print_section "🔧 Generating Bindings"

((total_tests++))
if run_test "Generate all bindings" "cargo run --bin generate-bindings --features generate-bindings"; then
    ((passed_tests++))
fi

# Build tests
print_section "🔨 Building Targets"

((total_tests++))
if run_test "Build WASM target" "cargo build --target wasm32-unknown-unknown --features wasm"; then
    ((passed_tests++))
fi

((total_tests++))
if run_test "Build Python target" "cargo build --features python"; then
    ((passed_tests++))
fi

# Generate WASM packages with wasm-pack (if available)
if command -v wasm-pack &> /dev/null; then
    ((total_tests++))
    if run_test "Generate wasm-pack bindings" "wasm-pack build --target bundler --out-dir wasm-test --features wasm"; then
        ((passed_tests++))
    fi
else
    echo -e "${YELLOW}⚠️  wasm-pack not found, skipping WASM package generation${NC}"
fi

# Run Rust integration tests
print_section "🦀 Rust Integration Tests"

((total_tests++))
if run_test "WASM binding tests" "cargo test --test wasm_tests"; then
    ((passed_tests++))
fi

((total_tests++))
if run_test "Python binding tests" "cargo test --test python_tests"; then
    ((passed_tests++))
fi

# Run JavaScript tests (if Node.js is available and wasm-pack succeeded)
if command -v node &> /dev/null && [ -d "wasm-test" ]; then
    print_section "🟨 JavaScript/WASM Tests"
    
    cd tests/bindings/js
    
    # Install dependencies
    ((total_tests++))
    if run_test "Install JS dependencies" "npm install"; then
        ((passed_tests++))
        
        # Run JS tests
        ((total_tests++))
        if run_test "JavaScript binding tests" "npm test"; then
            ((passed_tests++))
        fi
    fi
    
    cd ../../..
else
    echo -e "${YELLOW}⚠️  Node.js not found or WASM bindings not generated, skipping JavaScript tests${NC}"
fi

# Run Python tests (if Python 3 is available)
if command -v python3 &> /dev/null; then
    print_section "🐍 Python Binding Tests"
    
    ((total_tests++))
    if run_test "Python binding tests" "python3 tests/bindings/python/test_runner.py"; then
        ((passed_tests++))
    fi
else
    echo -e "${YELLOW}⚠️  Python 3 not found, skipping Python tests${NC}"
fi

# Compare results across platforms
print_section "🔬 Cross-Platform Comparison"

if [ -f "test-results-wasm.json" ] || [ -f "test-results-python.json" ]; then
    ((total_tests++))
    if run_test "Cross-platform comparison" "python3 tests/bindings/compare_results.py"; then
        ((passed_tests++))
    fi
else
    echo -e "${YELLOW}⚠️  No test results found for comparison${NC}"
fi

# Final summary
print_section "📊 Final Results"

echo "Tests passed: $passed_tests/$total_tests"
success_rate=$(( passed_tests * 100 / total_tests ))
echo "Success rate: $success_rate%"

if [ $passed_tests -eq $total_tests ]; then
    echo -e "${GREEN}🎉 All tests passed! Bindings are working correctly across all platforms.${NC}"
    exit 0
elif [ $success_rate -ge 80 ]; then
    echo -e "${YELLOW}⚠️  Most tests passed, but some issues detected.${NC}"
    exit 1
else
    echo -e "${RED}❌ Significant test failures detected. Check binding implementations.${NC}"
    exit 1
fi
