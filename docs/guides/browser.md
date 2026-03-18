# conda in the browser

conda-express includes **cx-wasm**, a WebAssembly build of the same rattler-based
solver and extractor used by the native `cx` CLI. Combined with the
`conda-emscripten` plugin, this enables real `conda install` to run entirely
client-side in a JupyterLite notebook — no server required.

This is not a reimplementation. The actual conda CLI (`conda.cli.main`) runs in
Python compiled to WASM, with cx-wasm replacing conda's native-code bottlenecks
(solver, extractor, repodata fetching) via conda's plugin API.

## Try it

Open the [live demo](https://jezdez.github.io/conda-express/demo/lab/index.html)
and run:

```python
%load_ext conda_emscripten
%conda install lz4
import lz4
```

Both `%conda` and `%cx` are available as IPython magics. All conda subcommands
work — `install`, `list`, `remove`, `search`, `info`, etc.

## How it works

```
Browser tab
  └── JupyterLite (main thread)
       └── cx-jupyterlite (rewrites bare "conda" → "%cx")
       └── xeus-python kernel (WebWorker)
            └── Python 3.13 (WASM/Emscripten)
                 └── conda (real, compiled to WASM)
                      └── cx-wasm plugins (Rust → WASM)
                           ├── solver: rattler/resolvo
                           ├── repodata: sharded fetch
                           └── extractor: .conda/.tar.bz2 → MEMFS
```

When you run `%conda install lz4`:

1. The magic parses the command, auto-injects `--yes`, snapshots `.so` files
2. Runtime patches are applied for MEMFS limitations (no `seek()`, no `subprocess`)
3. Real conda CLI runs — `CxWasmSolver` delegates to cx-wasm's Rust solver
4. Packages are downloaded (sync XHR), extracted (cx-wasm Rust extractor), and linked
5. New `.so` files are loaded via `ctypes.CDLL(RTLD_GLOBAL)` so C extensions work

## Packages

Packages come from [emscripten-forge](https://emscripten-forge.org/) — the same
packages as native conda, compiled to WebAssembly. Both pure Python packages
(from conda-forge) and C extension packages (from emscripten-forge) work.

## Local development

To build and test the JupyterLite demo locally:

```bash
# Prerequisites: pixi, wasm-pack
# Build the cx-wasm WASM module
pixi run -e web wasm-build

# Build the conda packages (cx-wasm-kernel, conda-emscripten)
pixi run -e recipes build-cx-wasm-kernel
pixi run -e recipes build-conda-emscripten-plugin

# Build and serve the JupyterLite site
pixi run -e lite lite-build-local
pixi run -e lite lite-serve
# Open http://localhost:8888/lab/index.html
```

The `--with-local` flag in `lite-build-local` adds the locally-built packages
from `output/` to the JupyterLite environment. Without it, only public
emscripten-forge packages are included.

## Components

| Component | Location | Role |
|---|---|---|
| cx-wasm | `crates/cx-wasm/` | Rust solver + extractor compiled to WASM |
| conda-emscripten | `conda-emscripten/` | conda plugin: solver, extractor, magics, patches |
| cx-jupyterlite | `cx-jupyterlite/` | JupyterLite extension: intercepts bare `conda` commands |
| cx-wasm-kernel | `recipes/cx-wasm-kernel/` | WASM files + Python bridge for xeus-python |
| JupyterLite site | `lite/` | Static site builder and demo notebooks |

## Limitations

- **MEMFS is volatile** — installed packages don't persist across page reloads
- **No subprocess** — post-link scripts are silently skipped
- **No symlinks or hardlinks** — MEMFS doesn't support them
- **Network required** — packages are fetched from emscripten-forge CDN at runtime
- **Platform** — only `emscripten-wasm32` packages from emscripten-forge are available
