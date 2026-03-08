/**
 * Bridge between cx-web streaming extract and Emscripten's MEMFS.
 *
 * cx-web runs in its own WASM module (wasm32-unknown-unknown) and streams
 * extracted files via callbacks. This module provides the `on_file` callback
 * that writes each file into Emscripten's in-memory filesystem (MEMFS) using
 * Module.FS.writeFile().
 */

/**
 * Subset of the Emscripten FS API we need.
 */
export interface EmscriptenFS {
  writeFile(path: string, data: Uint8Array | string): void;
  mkdir(path: string): void;
  analyzePath(path: string): { exists: boolean };
}

export interface MemfsWriterOptions {
  /** Root prefix in MEMFS where packages are installed (e.g. "/prefix") */
  prefix: string;
  /** Emscripten FS object (typically Module.FS or globalThis.Module.FS) */
  fs: EmscriptenFS;
  /** Optional: called for each file written, for logging/progress */
  onWrite?: (fullPath: string, size: number) => void;
}

/**
 * Ensure all parent directories exist in MEMFS.
 * Emscripten's FS.writeFile doesn't create intermediate dirs.
 */
function ensureParentDirs(fs: EmscriptenFS, filePath: string): void {
  const parts = filePath.split('/').filter(Boolean);
  let current = '';
  for (let i = 0; i < parts.length - 1; i++) {
    current += '/' + parts[i];
    if (!fs.analyzePath(current).exists) {
      fs.mkdir(current);
    }
  }
}

/**
 * Create an `on_file` callback for cx-web's streaming bootstrap that writes
 * files to Emscripten MEMFS.
 *
 * Usage with cx-web:
 * ```js
 * const onFile = createMemfsWriter({ prefix: '/prefix', fs: Module.FS });
 * await cx_bootstrap_streaming(lockfile, platform, onProgress, onFile);
 * ```
 */
export function createMemfsWriter(
  options: MemfsWriterOptions
): (packageName: string, path: string, bytes: Uint8Array) => void {
  const { prefix, fs, onWrite } = options;
  const normalizedPrefix = prefix.endsWith('/') ? prefix.slice(0, -1) : prefix;

  return (packageName: string, path: string, bytes: Uint8Array) => {
    const fullPath = `${normalizedPrefix}/${path}`;
    ensureParentDirs(fs, fullPath);
    fs.writeFile(fullPath, bytes);
    onWrite?.(fullPath, bytes.length);
  };
}
