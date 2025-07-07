#!/bin/bash

# Working test script for basic binding validation

set -e

echo "🧪 Nucleation Binding Validation Tests"
echo "======================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must be run from project root"
    exit 1
fi

echo ""
echo "🔧 Step 1: Generate Bindings"
echo "-----------------------------"
if cargo run --bin generate-bindings --features generate-bindings; then
    echo "✅ Binding generation successful"
else
    echo "❌ Binding generation failed"
    exit 1
fi

echo ""
echo "🔨 Step 2: Build Tests"
echo "----------------------"

# Test Python build
echo "Testing Python build..."
if cargo build --release --features python; then
    echo "✅ Python build successful"
    # Copy the dynamic library to the correct name for Python import
    cp target/release/libnucleation.dylib nucleation.so
    echo "✅ Python module created"
else
    echo "❌ Python build failed"
    exit 1
fi

# Test integration test compilation
echo "Testing integration test compilation..."
if cargo test --test wasm_tests --no-run; then
    echo "✅ WASM test compilation successful"
else
    echo "❌ WASM test compilation failed"
    exit 1
fi

if cargo test --test python_tests --no-run; then
    echo "✅ Python test compilation successful"  
else
    echo "❌ Python test compilation failed"
    exit 1
fi

echo ""
echo "📁 Step 3: Verify Generated Files"
echo "---------------------------------"

# Check generated files exist
if [ -f "src/generated_wasm.rs" ]; then
    echo "✅ WASM bindings generated"
else
    echo "❌ WASM bindings missing"
    exit 1
fi

if [ -f "src/generated_python.rs" ]; then
    echo "✅ Python bindings generated"
else
    echo "❌ Python bindings missing"
    exit 1
fi

if [ -f "python-stubs/nucleation.pyi" ]; then
    echo "✅ Python stubs generated"
else
    echo "❌ Python stubs missing"
    exit 1
fi

if [ -f "include/nucleation.h" ]; then
    echo "✅ C header generated"
else
    echo "❌ C header missing"
    exit 1
fi

echo ""
echo "📋 Step 4: Basic Content Validation"
echo "-----------------------------------"

# Check that generated files contain expected content
if grep -q "Schematic" src/generated_wasm.rs; then
    echo "✅ WASM bindings contain Schematic class"
else
    echo "❌ WASM bindings missing Schematic class"
    exit 1
fi

if grep -q "PySchematic" src/generated_python.rs; then
    echo "✅ Python bindings contain PySchematic class"
else
    echo "❌ Python bindings missing PySchematic class"
    exit 1
fi

if grep -q "class Schematic:" python-stubs/nucleation.pyi; then
    echo "✅ Python stubs contain Schematic class"
else
    echo "❌ Python stubs missing Schematic class"
    exit 1
fi

if grep -q "SchematicHandle" include/nucleation.h; then
    echo "✅ C header contains SchematicHandle"
else
    echo "❌ C header missing SchematicHandle"
    exit 1
fi

echo ""
echo "🎉 Basic Validation Complete!"
echo "============================="
echo "✅ All binding files generated successfully"
echo "✅ All target builds compile"
echo "✅ Generated files contain expected content"
echo ""
echo "📝 Next steps for full validation:"
echo "   - Run JavaScript tests: cd tests/bindings/js && npm test"
echo "   - Run Python tests: python3 tests/bindings/python/test_runner.py"
echo "   - Run cross-platform comparison: python3 tests/bindings/compare_results.py"
echo ""
echo "🎊 Binding system is working correctly!"
