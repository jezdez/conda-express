# PoC: conda install in the Browser

## Architecture

```
User: conda install numpy
        │
        ▼
┌─────────────────────────────────────────────────┐
│ conda-browser (Python, patched)                 │
│                                                 │
│  1. Parse spec ("numpy")                        │
│  2. Fetch repodata via requests (browser fetch) │
│  3. Call solver                                 │
│  4. Download packages via requests              │
│  5. Extract via cx-web bridge                   │
│  6. Link to prefix (copy-only on MEMFS)         │
└─────────────────────────────────────────────────┘
        │           │             │
        ▼           ▼             ▼
   ┌─────────┐ ┌──────────┐ ┌──────────────┐
   │ Solver  │ │ requests │ │ cx-web       │
   │ (WASM)  │ │ (fetch)  │ │ (streaming   │
   │ resolvo │ │          │ │  extract)    │
   └─────────┘ └──────────┘ └──────────────┘
```

## Solver: Option C — py-resolvo-conda

Build a minimal PyO3 extension for emscripten-wasm32 that exposes only:
- `rattler_solve` / `resolvo` (SAT solver)
- `rattler_conda_types` (package records, matchspecs)

No networking. No repodata gateway. No tokio async.

### API surface

```python
from py_resolvo_conda import solve

solution = solve(
    repodata={"channel_url": repodata_json_str, ...},
    specs=["numpy >=1.24"],
    installed=[...],  # current prefix records
    platform="emscripten-wasm32",
)
# solution: list of PackageRecord dicts to install/remove
```

### Build

```bash
maturin build \
    --target wasm32-unknown-emscripten \
    --release \
    --features solve-only
```

### Crate structure

```
crates/py-resolvo-conda/
├── Cargo.toml
├── pyproject.toml
└── src/
    └── lib.rs          # PyO3 bindings: solve(), parse_repodata(), etc.
```

Dependencies:
- `rattler_solve` (with resolvo backend)
- `rattler_conda_types`
- `pyo3` (with `abi3-py312`)
- NO `rattler_repodata_gateway`
- NO `reqwest`, `tokio`

### Known risks

1. `rattler_conda_types` uses `memmap2` — needs `#[cfg]` disable for WASM
2. `rattler_conda_types` optionally uses `rayon` — disable for WASM
3. `simd-json` may need feature-gating for WASM
4. PyO3 + emscripten is supported but has rough edges (linker flags)

## conda-rattler-solver integration

The existing `conda-rattler-solver` plugin calls py-rattler's gateway-based
solve. For the browser, we need a variant that:

1. Fetches repodata in Python (using `requests`, which uses browser fetch)
2. Passes raw JSON to py-resolvo-conda's `solve()`
3. Converts the solution back to conda's internal format

This could be:
- A patch to `conda-rattler-solver` (conditional import path)
- Or a new `conda-wasm-solver` plugin that uses py-resolvo-conda directly

## Package extraction

Two options:
1. **cx-web bridge**: Python calls JS which calls cx-web's streaming extract
   (already implemented in Rust/WASM, proven fast)
2. **Pure Python**: Use `tarfile` + `zipfile` (stdlib, slow but works)

Recommended: cx-web bridge for `.conda` and `.tar.bz2` formats.

## Linking

MEMFS does not support hard links or symlinks. Patch 004 (not yet written)
will force `LinkType.copy` when `sys.platform == "emscripten"`.

## Prerequisites

- [x] conda-browser package (noarch, patched)
- [ ] py-resolvo-conda WASM extension (~40-80h)
- [ ] conda-rattler-solver patch for WASM solve path
- [ ] 004-linking-memfs.patch
- [ ] Emscripten-wasm32 lockfile with all deps
- [ ] JupyterLite integration test
