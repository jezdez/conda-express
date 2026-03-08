/**
 * JupyterLite extension that bootstraps a conda environment via cx-web.
 *
 * On activation, this plugin:
 * 1. Loads the cx-web WASM module
 * 2. Streams all packages from the embedded lockfile to Emscripten MEMFS
 * 3. Writes conda config (.condarc, .cx.json) so conda works at runtime
 * 4. Reports progress to the console (UI progress can be wired separately)
 */

import { JupyterFrontEnd, JupyterFrontEndPlugin } from '@jupyterlab/application';
import { createMemfsWriter, type EmscriptenFS } from './memfs';
import { writeCondaConfig, writeCxMarker } from './conda-config';

export { createMemfsWriter, type EmscriptenFS, type MemfsWriterOptions } from './memfs';
export { writeCondaConfig, writeCxMarker, type CondaConfigOptions } from './conda-config';

const PLUGIN_ID = '@conda-express/jupyterlite-cx:bootstrap';
const PREFIX = '/prefix';

/**
 * Get the Emscripten FS object from the global scope.
 * In JupyterLite/xeus-python, Module.FS is available globally.
 */
function getEmscriptenFS(): EmscriptenFS | null {
  const g = globalThis as any;
  return g.Module?.FS ?? null;
}

/**
 * Dynamically import cx-web's WASM module.
 * The cx-web package must be available at runtime (bundled or served).
 */
async function loadCxWeb(): Promise<any> {
  // cx-web is expected to be bundled or available as a package
  // The exact import path depends on the deployment setup
  const mod = await import(/* webpackIgnore: true */ '../cx_web.js');
  await mod.default();
  return mod;
}

const plugin: JupyterFrontEndPlugin<void> = {
  id: PLUGIN_ID,
  autoStart: true,
  activate: async (app: JupyterFrontEnd) => {
    console.log(`[cx] ${PLUGIN_ID} activating...`);

    const fs = getEmscriptenFS();
    if (!fs) {
      console.warn('[cx] Emscripten FS not available — skipping bootstrap');
      return;
    }

    try {
      const cx = await loadCxWeb();
      const version = cx.cx_init();
      console.log(`[cx] loaded ${version}`);

      const embeddedLockfile = cx.cx_embedded_lockfile();
      const embeddedPlatform = cx.cx_embedded_platform();

      if (!embeddedLockfile || !embeddedPlatform) {
        console.warn(
          '[cx] no embedded lockfile/platform — build cx-web with CX_LOCKFILE and CX_PLATFORM'
        );
        return;
      }

      console.log(`[cx] bootstrapping ${embeddedPlatform} environment to ${PREFIX}...`);

      const onProgress = (current: number, total: number, name: string) => {
        console.log(`[cx] ${current + 1}/${total}: ${name}`);
      };

      const onFile = createMemfsWriter({
        prefix: PREFIX,
        fs,
      });

      const result = await cx.cx_bootstrap_streaming(
        embeddedLockfile,
        embeddedPlatform,
        onProgress,
        onFile
      );

      writeCondaConfig({ prefix: PREFIX, fs });
      writeCxMarker({
        prefix: PREFIX,
        fs,
        totalPackages: result.packages_installed,
        totalFiles: result.total_files,
        totalSize: result.total_size,
      });

      console.log(
        `[cx] bootstrap complete: ${result.packages_installed} packages, ` +
          `${result.total_files} files, ${(result.total_size / 1024 / 1024).toFixed(1)} MB`
      );

      if (result.errors.length > 0) {
        console.warn(`[cx] ${result.errors.length} errors:`, result.errors);
      }
    } catch (err) {
      console.error('[cx] bootstrap failed:', err);
    }
  },
};

export default plugin;
