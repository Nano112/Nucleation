# 🧬 Nucleation

**Nucleation** is a high-performance Minecraft schematic engine written in Rust — with full support for **Rust**, **WebAssembly/JavaScript**, **Python**, and **FFI-based integrations** like **PHP** and **C**.

> Built for performance, portability, and parity across ecosystems.

---

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)
[![CI/CD](https://github.com/Schem-at/Nucleation/workflows/Nucleation%20CI%2FCD/badge.svg)](https://github.com/Schem-at/Nucleation/actions)

---

## ✨ Features

- ✅ **Multi-format support**: `.schematic`, `.litematic`, `.nbt`, etc.
- 🧠 **Memory-safe Rust core** with zero-copy deserialization
- 🌐 **WASM module** for browser + Node.js with TypeScript support
- 🐍 **Native Python bindings** (`pip install nucleation`)
- ⚙️ **C-compatible FFI** for PHP, C, Go, etc.
- 🎨 **Blockpedia integration** for color analysis and block transformations (native targets)
- 🔄 **Feature parity** across all interfaces via single API definition
- 📦 **Binary builds** for Linux, macOS, Windows (x86_64 + ARM64)
- 🚀 **Automatic binding generation** from centralized API definitions
- 🧪 **Comprehensive test suite** with CI/CD pipeline

---

## 📦 Installation

### 🔧 Rust

```bash
cargo add nucleation
````

### 🌐 JavaScript / TypeScript (WASM)

```bash
npm install nucleation
```

### 🐍 Python

```bash
pip install nucleation
```

### 🧩 C / PHP / FFI

Download prebuilt `.so` / `.dylib` / `.dll` from [Releases](https://github.com/Schem-at/Nucleation/releases)
or build locally using:

```bash
./build-ffi.sh
```

---

## 🚀 Quick Examples

### Rust

```rust
use nucleation::UniversalSchematic;

let bytes = std::fs::read("example.litematic")?;
let mut schematic = UniversalSchematic::new("my_schematic");
schematic.load_from_data(&bytes)?;
println!("{:?}", schematic.get_info());
```

📖 → [Documentation](examples/rust.md) | [Complete Code Example](examples/rust_example.rs)

---

### JavaScript (WASM)

```ts
import { SchematicParser } from "nucleation";

const bytes = await fetch("example.litematic").then(r => r.arrayBuffer());
const parser = new SchematicParser();
await parser.fromData(new Uint8Array(bytes));

console.log(parser.getDimensions());
```

📖 → [Documentation](examples/wasm.md) | [Complete Code Example](examples/wasm_example.js)

---

### Python

```python
from nucleation import Schematic

with open("example.litematic", "rb") as f:
    data = f.read()

schem = Schematic("my_schematic")
schem.load_from_bytes(data)

print(schem.get_info())
```

📖 → [Documentation](examples/python.md) | [Complete Code Example](examples/python_example.py)

---

### FFI (PHP/C)

```c
#include "nucleation.h"

SchematicHandle* handle = schematic_new("MySchem");
schematic_load_data(handle, data_ptr, data_len);

CSchematicInfo info;
schematic_get_info(handle, &info);
printf("Size: %dx%dx%d\n", info.width, info.height, info.depth);

schematic_free(handle);
```

📖 → [Documentation](examples/ffi.md) | [Complete Code Example](examples/ffi_example.c)

---

## 🔧 Development

### Building

```bash
# Build the Rust core
cargo build --release

# Build WASM module with target support
cargo build --target wasm32-unknown-unknown --features wasm
./build-wasm.sh

# Build Python bindings locally
maturin develop --features python

# Build FFI libs
./build-ffi.sh
```

### 🤖 Automated Binding Generation

Nucleation uses a **single source of truth** approach for all bindings. The API is defined once in `src/api_definition.rs` and automatically translated to all supported languages:

```bash
# Generate all binding files from API definition
cargo run --bin generate-bindings

# Check if bindings are up to date
cargo run --bin generate-bindings check

# Generate API documentation report
cargo run --bin generate-bindings report
```

This generates:
- **WASM**: TypeScript definitions and JavaScript bindings
- **Python**: PyO3 bindings and `.pyi` type stubs  
- **FFI**: C header files and Rust FFI implementations

### 🧪 Testing

```bash
# Run all tests
cargo test

# Test specific targets
cargo test --features wasm --test wasm_tests        # WASM tests
cargo test --test python_tests                      # Python tests
cargo test --test blockpedia_integration_test       # Blockpedia tests (non-WASM only)

# Test WASM build specifically
cargo build --target wasm32-unknown-unknown --features wasm
```

**Note**: WASM builds exclude blockpedia features for compatibility. Color analysis and block transformations are available only on native targets.

### Version Management

Versions are centrally managed in `version.toml`. Use the Makefile commands:

```bash
# Check version consistency across all files
make version-check

# Bump versions automatically
make version-bump-patch    # 0.1.0 → 0.1.1
make version-bump-minor    # 0.1.0 → 0.2.0  
make version-bump-major    # 0.1.0 → 1.0.0

# Update all files from version.toml
make version-update
```

📖 → [Full Version Management Guide](docs/VERSION_MANAGEMENT.md)

---

## 📚 Submodules & Bindings

### 📄 Documentation & Examples

Each binding includes comprehensive documentation and working code examples:

| Language | API Documentation | Working Example | Type Definitions |
|----------|------------------|-----------------|------------------|
| **Rust** | [`examples/rust.md`](examples/rust.md) | [`examples/rust_example.rs`](examples/rust_example.rs) | Native Rust docs |
| **JavaScript/WASM** | [`examples/wasm.md`](examples/wasm.md) | [`examples/wasm_example.js`](examples/wasm_example.js) | [`pkg/nucleation.d.ts`](pkg/nucleation.d.ts) |
| **Python** | [`examples/python.md`](examples/python.md) | [`examples/python_example.py`](examples/python_example.py) | [`python-stubs/nucleation.pyi`](python-stubs/nucleation.pyi) |
| **C/FFI** | [`examples/ffi.md`](examples/ffi.md) | [`examples/ffi_example.c`](examples/ffi_example.c) | [`include/nucleation.h`](include/nucleation.h) |


---

## ⚖️ License

Licensed under the **GNU AGPL-3.0-only**.
See [`LICENSE`](./LICENSE) for full terms.


Made by [@Nano112](https://github.com/Nano112) with ❤️