## 0 · What gets published

After running the provided build script you’ll find **`pkg/`** with these key files:

| File                            | Why it exists                                                                               |
| ------------------------------- | ------------------------------------------------------------------------------------------- |
| `nucleation_bg.wasm`            | Compiled WebAssembly binary.                                                                |
| `nucleation-original.js`        | Raw `wasm-pack` ES module (expects you to pass the `.wasm` URL/bytes).                      |
| `nucleation.js`                 | **Universal wrapper** (auto-detects Node vs browser & fetches the `.wasm` for you).         |
| `nucleation-cdn-loader.js`      | Tiny wrapper that always resolves the correct relative `.wasm` path when served from a CDN. |
| `nucleation.d.ts` & `*_bg.d.ts` | TypeScript typings.                                                                         |
| `package.json` (rewritten)      | Exports map points the world at `nucleation.js` by default, or `cdn-loader` for CDN users.  |

---

## 1 · Loading the module (three ways)

### 1.1  Bundlers **or** Node (automatic)

```js
import init, { SchematicWrapper } from "nucleation";   // npm install nucleation
await init();                                          // auto-detects env & fetches WASM
const sch = new SchematicWrapper();
```

`init()` can also accept **bytes** or a **URL** if you want full control.

### 1.2  Browser via CDN

```html
<script type="module">
  import init, { SchematicWrapper } from
    "https://cdn.jsdelivr.net/npm/nucleation@latest/nucleation-cdn-loader.js";

  await init();                // resolves ./nucleation_bg.wasm next to the .js
  const sch = new SchematicWrapper();
</script>
```

### 1.3  Advanced manual loading

```js
import init, { SchematicWrapper } from "nucleation";
const bytes = await fetch("/path/my.wasm").then(r => r.arrayBuffer());
await init(bytes);
```

---

## 2 · Runtime side-effects

* The **first** thing the module does (via `#[wasm_bindgen(start)]`) is:

```text
Initializing schematic utilities
```

printed to `console.log`.

---

## 3 · API surface (JavaScript)

Nothing here changed, but for completeness:

<details>
<summary>Click to expand full function map</summary>

### 3.1 `SchematicWrapper`

| Method                            | JS Signature                          | Purpose                                                                                          |                             |
| --------------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------ | --------------------------- |
| **Constructor**                   | `new SchematicWrapper()`              | Empty schematic named **“Default”**.                                                             |                             |
| `from_data`                       | `(bytes: Uint8Array) → void`          | Auto-detect `.litematic` or WorldEdit `.schematic`.                                              |                             |
| `from_litematic` / `to_litematic` | `(bytes) → void` / `() → Uint8Array`  | Explicit Litematic.                                                                              |                             |
| `from_schematic` / `to_schematic` | same                                  | Explicit WorldEdit.                                                                              |                             |
| `set_block`                       | `(x,y,z, blockName)`                  | Quick place, no props.                                                                           |                             |
| `set_block_with_properties`       | `(x,y,z, blockName, propsObj)`        | Props as plain JS object.                                                                        |                             |
| `set_block_from_string`           | `(x,y,z, fullString)`                 | Parses `[props]{nbt}` + barrel `{signal=n}` sugar.                                               |                             |
| `copy_region`                     | `(src, min..max, target, excluded[])` | Copies cuboid, skips listed block types.                                                         |                             |
| `get_block`                       | `(x,y,z) → string?`                   | Name only.                                                                                       |                             |
| `get_block_with_properties`       | `→ BlockStateWrapper?`                | Full state.                                                                                      |                             |
| `get_block_entity`                | \`→ object                            | null\`                                                                                           | Converts NBT to JS objects. |
| `get_all_block_entities`          | `→ Array<object>`                     |                                                                                                  |                             |
| `print_schematic`                 | `() → string`                         | ASCII preview.                                                                                   |                             |
| `debug_info`                      | `() → string`                         | Name + region count.                                                                             |                             |
| `get_dimensions`                  | `() → [x,y,z]`                        |                                                                                                  |                             |
| `get_block_count` / `get_volume`  | `() → number`                         |                                                                                                  |                             |
| `get_region_names`                | `() → string[]`                       |                                                                                                  |                             |
| `blocks`                          | `() → Array`                          | Each `{x,y,z,name,properties}`.                                                                  |                             |
| `chunks`                          | `(w,h,l) → Array`                     | Returns bottom-up ordered chunks.                                                                |                             |
| `chunks_with_strategy`            | `(w,h,l,strat,cx,cy,cz) → Array`      | Strategies: `"distance_to_camera"`, `"top_down"`, `"bottom_up"`, `"center_outward"`, `"random"`. |                             |
| `get_chunk_blocks`                | `(offX,offY,offZ,w,h,l) → Array`      | Arbitrary cuboid slice.                                                                          |                             |

### 3.2 `BlockStateWrapper`

| Method                                                     | Purpose |
| ---------------------------------------------------------- | ------- |
| **Constructor** `new BlockStateWrapper("minecraft:stone")` |         |
| `with_property(key,val)` – mutates & returns `void`.       |         |
| `name()` – *string*                                        |         |
| `properties()` – plain JS object                           |         |

### 3.3 Standalone helpers

| Function                    | Returns                  |
| --------------------------- | ------------------------ |
| `debug_schematic(sch)`      | Pretty ASCII + header.   |
| `debug_json_schematic(sch)` | Header + full JSON dump. |

</details>

---

## 4 · Typical usage snippet

```js
import init, { SchematicWrapper } from "nucleation";
await init();                       // works everywhere

const sch = new SchematicWrapper();
sch.set_block(0, 0, 0, "minecraft:stone");
sch.set_block_from_string(1, 0, 0,
  'minecraft:barrel[facing=up]{signal=13}'
);

console.log(sch.print_schematic());

// Download as .litematic
const blob = new Blob([sch.to_litematic()], { type: "application/octet-stream" });
Object.assign(document.createElement("a"), {
  href: URL.createObjectURL(blob),
  download: "build.litematic"
}).click();
```

---

### Final notes

* **Universal wrapper** (`nucleation.js`) hides environment quirks—use it unless you **must** supply your own bytes.
* The `"random"` chunk strategy is deterministic: it hashes the schematic name for repeatable shuffles.
* `excluded_blocks` and `properties` **must** be *plain* JS arrays/objects—`Map`, `Set`, etc. will throw.

Happy scheming on the web 🛠️✨
