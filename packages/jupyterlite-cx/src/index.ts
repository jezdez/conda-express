/**
 * JupyterLite extension that bootstraps a conda environment via cx-wasm.
 *
 * In JupyterLite (xeus-python) the kernel runs in its own WebWorker; packages
 * are pre-installed at build time by jupyterlite-xeus from environment.yml.
 * This plugin handles the remaining setup:
 *   • In classic JupyterLab / JupyterLab Desktop: streams packages to MEMFS.
 *   • In JupyterLite: conda packages are pre-installed; the plugin logs status.
 */

import { JupyterFrontEnd, JupyterFrontEndPlugin } from '@jupyterlab/application';
import { createMemfsWriter, type EmscriptenFS } from './memfs';
import { writeCondaConfig, writeCxMarker } from './conda-config';

export { createMemfsWriter, type EmscriptenFS, type MemfsWriterOptions } from './memfs';
export { writeCondaConfig, writeCxMarker, type CondaConfigOptions } from './conda-config';

const PLUGIN_ID = '@conda-express/jupyterlite-cx:bootstrap';
const PREFIX = '/prefix';

/**
 * Return the Emscripten FS from the global scope, if available.
 * Works in classic JupyterLab Desktop / embedded WASM setups, but returns
 * null when running as a JupyterLab extension in the main thread of
 * JupyterLite (the kernel FS lives in a separate WebWorker there).
 */
function getEmscriptenFS(): EmscriptenFS | null {
  const g = globalThis as Record<string, any>;
  return g['Module']?.['FS'] ?? null;
}

/**
 * Dynamically import the cx-wasm module from its runtime location.
 * The .wasm file must be co-located with the extension's static assets.
 */
async function loadCxWasm(): Promise<Record<string, any>> {
  const mod = await import(/* webpackIgnore: true */ '../cx_wasm.js');
  await (mod as any)['default']();
  return mod as Record<string, any>;
}

const plugin: JupyterFrontEndPlugin<void> = {
  id: PLUGIN_ID,
  autoStart: true,
  activate: async (app: JupyterFrontEnd) => {
    console.log(`[cx] ${PLUGIN_ID} activating…`);

    const fs = getEmscriptenFS();

    if (!fs) {
      // In JupyterLite the kernel FS is inside the xeus-python WebWorker.
      // Packages are pre-installed by jupyterlite-xeus at build time using
      // the environment.yml alongside the JupyterLite configuration.
      const isLite = (globalThis as Record<string, any>)['jupyterapp']
        ?.name?.includes('lite');
      if (isLite) {
        console.log(
          '[cx] Running in JupyterLite — conda packages were pre-installed ' +
            'at build time via jupyterlite-xeus environment.yml. ' +
            'Runtime conda install requires the cx-worker bridge (future work).'
        );
      } else {
        console.warn(
          '[cx] Emscripten FS not found — cx-wasm bootstrap skipped. ' +
            'Module.FS must be available in globalThis for MEMFS bootstrap.'
        );
      }
      return;
    }

    try {
      const cx = await loadCxWasm();
      const version: string = cx['cx_init']();
      console.log(`[cx] loaded ${version}`);

      const embeddedLockfile: string | null = cx['cx_embedded_lockfile']();
      const embeddedPlatform: string | null = cx['cx_embedded_platform']();

      if (!embeddedLockfile || !embeddedPlatform) {
        console.warn(
          '[cx] No embedded lockfile/platform — build cx-wasm with ' +
            'CX_LOCKFILE and CX_PLATFORM env vars set.'
        );
        return;
      }

      console.log(`[cx] bootstrapping ${embeddedPlatform} → ${PREFIX}`);

      const onProgress = (current: number, total: number, name: string) => {
        console.log(`[cx]  ${current + 1}/${total} ${name}`);
      };

      const onFile = createMemfsWriter({ prefix: PREFIX, fs });

      const result = await cx['cx_bootstrap_streaming'](
        embeddedLockfile,
        embeddedPlatform,
        onProgress,
        onFile
      );

      writeCondaConfig({ prefix: PREFIX, fs });
      writeCxMarker({
        prefix: PREFIX,
        fs,
        totalPackages: result['packages_installed'],
        totalFiles: result['total_files'],
        totalSize: result['total_size'],
      });

      console.log(
        `[cx] bootstrap complete: ${result['packages_installed']} packages, ` +
          `${result['total_files']} files, ` +
          `${(result['total_size'] / 1024 / 1024).toFixed(1)} MB`
      );

      if (result['errors']?.length > 0) {
        console.warn(`[cx] ${result['errors'].length} errors:`, result['errors']);
      }
    } catch (err) {
      console.error('[cx] bootstrap failed:', err);
    }
  },
};

export default plugin;
