# Nucleation Binding Validation Test Suite

This directory contains comprehensive tests to validate that your generated bindings work correctly across all export targets (WASM, Python, FFI) and maintain feature parity.

## Overview

The test suite ensures that:
- All binding targets can be built successfully
- Generated bindings provide the same API surface
- Core functionality works identically across platforms
- Performance characteristics are reasonable
- Error handling is consistent

## Test Structure

```
tests/bindings/
├── mod.rs                      # Test module definitions and shared utilities
├── shared_test_cases.rs        # Common test logic for cross-platform validation
├── wasm_tests.rs              # Rust integration tests for WASM bindings
├── python_tests.rs            # Rust integration tests for Python bindings
├── js/                        # JavaScript/WASM test runner
│   ├── test_runner.js         # Node.js test suite for WASM bindings
│   └── package.json           # Dependencies and scripts
├── python/                    # Python test runner
│   └── test_runner.py         # Python test suite for Python bindings
├── compare_results.py         # Cross-platform result comparison tool
├── run_all_tests.sh          # Master test runner script
└── README.md                 # This file
```

## Quick Start

### Run All Tests

The easiest way to run the complete test suite:

```bash
# From project root
./tests/bindings/run_all_tests.sh
```

This will:
1. Generate bindings for all targets
2. Build all targets
3. Run platform-specific tests
4. Compare results across platforms
5. Generate a comprehensive report

### Individual Test Runners

#### WASM/JavaScript Tests

```bash
# Generate WASM bindings first
cargo run --bin generate-bindings -- wasm
wasm-pack build --target bundler --out-dir wasm-test --features wasm

# Run JavaScript tests
cd tests/bindings/js
npm install
npm test
```

#### Python Tests

```bash
# Generate Python bindings first
cargo run --bin generate-bindings -- python
cargo build --features python

# Run Python tests
python3 tests/bindings/python/test_runner.py
```

#### Rust Integration Tests

```bash
# Test WASM binding generation
cargo test --test wasm_tests

# Test Python binding generation  
cargo test --test python_tests

# Run ignored tests (requires external dependencies)
cargo test --test wasm_tests -- --ignored
cargo test --test python_tests -- --ignored
```

### Cross-Platform Comparison

After running tests on multiple platforms:

```bash
python3 tests/bindings/compare_results.py
```

This generates a detailed comparison report showing:
- Consistency across platforms
- Performance differences
- Failed tests and their causes
- Recommendations for improvement

## Test Categories

### 1. Binding Generation Tests
- Verify bindings can be generated successfully
- Check that all API elements are included
- Validate generated code syntax

### 2. Build Tests
- Ensure all targets compile without errors
- Verify feature flags work correctly
- Check for missing dependencies

### 3. Functional Tests
- **Schematic Creation**: Create new schematics and verify properties
- **File Loading**: Load litematic and schematic files
- **Format Conversion**: Convert between formats while preserving data
- **Block Operations**: Set/get blocks with properties
- **BlockState Manipulation**: Create and modify block states
- **Free Functions**: Test module-level functions
- **String Representations**: Verify `__str__` and `__repr__` methods (Python)

### 4. Cross-Platform Validation
- Compare identical operations across platforms
- Verify consistent results and error handling
- Check performance characteristics
- Validate API parity

## Test Data

Tests use sample files from `tests/samples/`:
- `1x1.litematic` - Minimal test case
- `sample.litematic` - Standard test schematic
- `sample.schem` - Standard schematic format
- Other sample files for comprehensive testing

## Dependencies

### Required
- Rust with `wasm32-unknown-unknown` target
- Python 3.7+
- Node.js 18+ (for JavaScript tests)

### Optional
- `wasm-pack` (for WASM package generation)
- npm (for JavaScript dependency management)

## Adding New Tests

### For All Platforms

1. Add test logic to `shared_test_cases.rs`
2. Update the API definition in `src/api_definition.rs` if needed
3. Add platform-specific implementations:
   - JavaScript: `js/test_runner.js`
   - Python: `python/test_runner.py`

### For Specific Platforms

#### WASM/JavaScript
Add test functions to `WasmTests` class in `js/test_runner.js`

#### Python
Add test methods to `PythonTests` class in `python/test_runner.py`

#### Rust Integration
Add tests to `wasm_tests.rs` or `python_tests.rs`

## Continuous Integration

The test suite is designed to work in CI environments:

```yaml
# Example GitHub Actions snippet
- name: Run binding tests
  run: |
    # Install dependencies
    rustup target add wasm32-unknown-unknown
    npm install -g wasm-pack
    
    # Run test suite
    ./tests/bindings/run_all_tests.sh
```

## Troubleshooting

### Common Issues

**WASM tests fail**: 
- Ensure `wasm32-unknown-unknown` target is installed
- Install `wasm-pack`: `cargo install wasm-pack`

**Python tests fail**:
- Check Python version (3.7+ required)
- Ensure PyO3 dependencies are available
- Verify generated bindings are importable

**Cross-platform comparison shows inconsistencies**:
- Check API definition completeness
- Review platform-specific implementation differences
- Verify test data consistency

### Debug Mode

Run tests with verbose output:

```bash
# JavaScript
cd tests/bindings/js && npm run test:verbose

# Python with detailed tracebacks
python3 tests/bindings/python/test_runner.py --verbose

# Rust with output
cargo test --test wasm_tests -- --nocapture
```

## Performance Benchmarking

The test suite includes basic performance metrics:
- Execution time for each test
- Memory usage estimates
- Cross-platform performance comparison

For detailed profiling, use platform-specific tools:
- JavaScript: Node.js profiler
- Python: `cProfile`
- Rust: `criterion` benchmarks

## Contributing

When adding new features to the binding system:

1. Update the API definition
2. Add corresponding tests to all platforms
3. Run the full test suite to ensure parity
4. Update this README if new test categories are added

The goal is to maintain 100% API parity across all binding targets.
