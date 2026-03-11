/**
 * cx-bootstrap.js — Thin Comlink client for the cx-worker Web Worker.
 *
 * Creates a Worker that owns all heavy computation (WASM, pyjs, bootstrap,
 * Python/conda execution) and exposes an async API via Comlink RPC.
 * Streaming events (logs, progress, stdout/stderr) arrive as postMessage
 * side-channel events and are routed through mutable handler callbacks.
 *
 * Usage:
 *   import { createCondaWorker } from './cx-bootstrap.js';
 *   const conda = await createCondaWorker({
 *       onLog: (msg, level) => console.log(msg),
 *       onProgress: (current, total, name) => updateBar(current, total),
 *       onPrint: (text) => appendOutput(text),
 *       onError: (text) => appendOutput(text),
 *   });
 *   const result = await conda.bootstrap({ lockfileUrl: '/conda-emscripten.lock' });
 *   await conda.runPythonSync('import conda; print(conda.__version__)');
 */

import * as Comlink from './vendor/comlink.mjs';

/**
 * Create a conda Web Worker and return a handle with RPC methods.
 *
 * @param {object} [callbacks]
 * @param {Function} [callbacks.onLog]      - (msg: string, level: string) => void
 * @param {Function} [callbacks.onProgress] - (current: number, total: number, name: string) => void
 * @param {Function} [callbacks.onPrint]    - (text: string) => void
 * @param {Function} [callbacks.onError]    - (text: string) => void
 * @returns {Promise<CondaWorkerHandle>}
 */
export async function createCondaWorker({ onLog, onProgress, onPrint, onError } = {}) {
    const workerUrl = new URL('./cx-worker.js', import.meta.url);
    const worker = new Worker(workerUrl);

    // Mutable handlers — callers can reassign to redirect output
    const handlers = { onLog, onProgress, onPrint, onError };

    // Route side-channel events from the Worker
    worker.addEventListener('message', (e) => {
        const d = e.data;
        if (!d || !d.__cx) return;
        switch (d.type) {
            case 'log':      handlers.onLog?.(d.msg, d.level); break;
            case 'progress': handlers.onProgress?.(d.current, d.total, d.name); break;
            case 'print':    handlers.onPrint?.(d.text); break;
            case 'error':    handlers.onError?.(d.text); break;
        }
    });

    // Wait for the Worker to signal that Comlink API is exposed
    await new Promise((resolve) => {
        const onReady = (e) => {
            if (e.data?.__cx && e.data.type === 'ready') {
                worker.removeEventListener('message', onReady);
                resolve();
            }
        };
        worker.addEventListener('message', onReady);
    });

    const api = Comlink.wrap(worker);

    return {
        /** Raw Worker instance for advanced event routing */
        worker,

        /** Mutable event handlers — reassign to redirect output per-operation */
        handlers,

        /**
         * Run the full bootstrap sequence in the Worker.
         * @param {object} opts
         * @param {string}   opts.lockfileUrl
         * @param {string}   [opts.platform]
         * @param {number[]} [opts.pythonVersion]
         * @param {string}   [opts.prefix]
         * @param {string[]} [opts.channels]
         * @param {string}   [opts.solver]
         * @param {string}   [opts.packageBaseUrl]
         * @param {boolean}  [opts.useCache]
         * @param {boolean}  [opts.forceRefresh]
         */
        bootstrap: (opts) => api.bootstrap(opts),

        /** Run Python code synchronously (exec). Output via print/error events. */
        runPythonSync: (code) => api.runPythonSync(code),

        /** Run Python code async (async_exec_eval). Returns the result value. */
        runPython: (code) => api.runPython(code),

        /** Clear the IndexedDB bootstrap cache. */
        clearCache: () => api.clearCache(),

        /** Get current Worker state: { ready, fromCache }. */
        getState: () => api.getState(),

        /** Terminate the Worker. */
        terminate: () => worker.terminate(),
    };
}
