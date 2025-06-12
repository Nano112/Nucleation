## 0 · Building with the FFI feature

```bash
# Build a cdylib that exposes the two functions below
cargo build --release --features ffi
# → target/release/libnucleation.{so|dll|dylib}
```

The crate must be compiled with **`--features ffi`** or these symbols will not be exported.

---

## 1 · Exposed symbols

| C name                 | Signature (C)                                                | Rust origin                                                                                                                          | What it does                                                                                                                 | Memory ownership |
| ---------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| `schematic_debug_info` | `char* schematic_debug_info(const UniversalSchematic* sch);` | Returns `CString` built from a short status string such as `"Schematic has 5 regions"` or `"null schematic"` if the pointer is null. | **Caller takes ownership** of the returned `char*` and must free it with `free()` *(C)* **or** `CString::from_raw` *(Rust)*. |                  |
| `print_debug_info`     | `void print_debug_info(const UniversalSchematic* sch);`      | Convenience wrapper that just calls `schematic_debug_info()`, prints the message to `stdout`, and **immediately frees the string**.  | No heap to free on the caller side.                                                                                          |                  |

---

## 2 · Usage examples

### 2.1  From plain C

```c
#include <stdio.h>
#include <stdlib.h>

// forward-decls (usually come from a generated binding header)
extern char* schematic_debug_info(const void* sch);
extern void  print_debug_info(const void* sch);

int main(void) {
    /* imagine you obtained a UniversalSchematic* from Rust
       via some extra constructor function */
    void* my_schematic = get_schematic_from_rust();

    // Option 1: fire-and-forget print
    print_debug_info(my_schematic);

    // Option 2: get the message
    char* msg = schematic_debug_info(my_schematic);
    printf("DEBUG (C side): %s\n", msg);
    free(msg);                           // YOU must free it!
    return 0;
}
```

### 2.2  From Rust (another crate)

```rust
#[link(name = "nucleation")]           // or the dynamic lib name on your target
extern "C" {
    fn schematic_debug_info(ptr: *const UniversalSchematic) -> *mut std::os::raw::c_char;
}

let sch: *const UniversalSchematic = obtain_schematic();
unsafe {
    let c_str = schematic_debug_info(sch);
    if !c_str.is_null() {
        println!("Rust FFI -> {}", std::ffi::CStr::from_ptr(c_str).to_string_lossy());
        // Convert back into CString to reclaim the allocation:
        let _ = CString::from_raw(c_str);
    }
}
```

---

## 3 · Important notes & gotchas

* **Thread safety** – The two functions are thread-safe as long as you never mutate the same `UniversalSchematic` from multiple threads without proper locking.
* **Null‐checking** – Both functions guard against `NULL` and produce a safe fallback string or message.
* **No constructors/destructors exported** – Your snippet only exposes *debug* helpers. In real code you’ll need additional `extern "C"` functions to create/destroy `UniversalSchematic` instances, or capture pointers produced elsewhere in Rust.

