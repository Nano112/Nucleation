# 🧬 Nucleation

**Nucleation** is a high-performance Minecraft schematic engine written in Rust — with full support for **Rust**, **WebAssembly/JavaScript**, **Python**, and **FFI-based integrations** like **PHP** and **C**.

> Built for performance, portability, and parity across ecosystems.

---

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)

---

## ✨ Features

- ✅ Multi-format support: `.schematic`, `.litematic`, `.nbt`, etc.
- 🧠 Memory-safe Rust core with zero-copy deserialization
- 🌐 WASM module for browser + Node.js
- 🐍 Native Python bindings (`pip install nucleation`)
- ⚙️ C-compatible FFI for PHP, C, Go, etc.
- 🔄 Feature parity across all interfaces
- 📦 Binary builds for Linux, macOS, Windows (x86_64 + ARM64)
- 🧱 Seamless integration with [Cubane](https://github.com/Nano112/cubane)

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

📖 → [More in `examples/rust.md`](examples/rust.md)

---

### JavaScript (WASM)

```ts
import { SchematicParser } from "nucleation";

const bytes = await fetch("example.litematic").then(r => r.arrayBuffer());
const parser = new SchematicParser();
await parser.fromData(new Uint8Array(bytes));

console.log(parser.getDimensions());
```

📖 → [More in `examples/wasm.md`](examples/wasm.md)

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

📖 → [More in `examples/python.md`](examples/python.md)

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

📖 → [More in `examples/ffi.md`](examples/ffi.md)

---

## 🔧 Development

```bash
# Build the Rust core
cargo build --release

# Build WASM module
./build-wasm.sh

# Build Python bindings locally
maturin develop --features python

# Build FFI libs
./build-ffi.sh
```

---

## 📚 Submodules & Bindings

### Rust
* [`examples/rust.md`](examples/rust.md)

### JavaScript/TypeScript
* [`examples/wasm.md`](examples/wasm.md)

### Python
* [`examples/python.md`](examples/python.md)

### FFI (C/PHP)
* [`examples/ffi.md`](examples/ffi.md)

---

## ⚖️ License

Licensed under the **GNU AGPL-3.0-only**.
See [`LICENSE`](./LICENSE) for full terms.


Made by [@Nano112](https://github.com/Nano112) with ❤️