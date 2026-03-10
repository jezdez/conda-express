/**
 * cx-bootstrap.js — Reusable conda-in-browser bootstrap module.
 *
 * Orchestrates the full sequence of loading cx-wasm, creating a pyjs runtime,
 * streaming packages from a lockfile into Emscripten MEMFS, initialising the
 * Python interpreter, and configuring conda for use.
 *
 * Usage:
 *   import { bootstrapConda } from './cx-bootstrap.js';
 *   const { pyjsModule, cxWasm } = await bootstrapConda({ lockfileUrl: '/conda-emscripten.lock', ... });
 */

import init, {
    cx_init, cx_bootstrap_streaming, cx_bootstrap_plan,
    cx_extract_package, cx_solve_init, cx_solve
} from './cx_wasm.js';

// ─── Utility exports ───────────────────────────────────────────────────────────

/**
 * Rewrite lockfile URLs from local file:// paths to a given HTTP base URL.
 * The generated pixi.lock contains absolute file:// paths pointing at the
 * local rattler-build output directory. At runtime in the browser we need
 * to redirect those to wherever the .conda packages are being served from.
 */
export function rewriteLockfileUrls(text, packageBaseUrl) {
    let result = text;
    const match = result.match(/url:\s*file:\/\/\/(\S+)/);
    if (match) {
        const captured = match[1].replace(/\/$/, '');
        const absPath = '/' + captured;
        const base = packageBaseUrl.replace(/\/$/, '');
        result = result.replaceAll('file://' + absPath, base);
        result = result.replaceAll(absPath + '/noarch/', base + '/');
    }
    return result;
}

/**
 * Recursively create directories in an Emscripten FS.
 */
export function ensureDir(FS, path) {
    const parts = path.split('/').filter(Boolean);
    let cur = '';
    for (const p of parts) {
        cur += '/' + p;
        try { FS.mkdir(cur); } catch (_) { /* exists */ }
    }
}

/**
 * Run Python code through a pyjs module instance and return the result.
 * Throws on error.
 */
export async function runPython(pyjsModule, code) {
    return await pyjsModule['async_exec_' + 'eval'](code);
}

/**
 * Extract a human-readable message from various error types (Error, pyjs
 * proxy objects, JsValue, etc.).
 */
export function formatPyError(e) {
    if (e instanceof Error) return e.message;
    if (typeof e === 'string') return e;
    try {
        if (e && typeof e._getattr === 'function') {
            return e._getattr('__str__')();
        }
    } catch (_) {}
    try {
        const s = e.toString();
        if (s !== '[object Object]') return s;
    } catch (_) {}
    try { return JSON.stringify(e); } catch (_) {}
    return String(e);
}

// ─── Main bootstrap ────────────────────────────────────────────────────────────

/**
 * Bootstrap a full conda-in-browser environment.
 *
 * @param {object} options
 * @param {string}   options.lockfileUrl    - URL to fetch the pixi lockfile from
 * @param {string}   [options.platform]     - Target platform (default: "emscripten-wasm32")
 * @param {number[]} [options.pythonVersion] - [major, minor] (default: [3, 13])
 * @param {string}   [options.prefix]       - Filesystem prefix (default: "")
 * @param {string[]} [options.channels]     - Channel URLs for .condarc
 * @param {string}   [options.solver]       - Solver name (default: "emscripten")
 * @param {string}   [options.packageBaseUrl] - Base URL for rewriting file:// lockfile paths
 * @param {Function} [options.onProgress]   - (current, total, name) => void
 * @param {Function} [options.onLog]        - (message, level) => void
 * @param {Function} [options.pyjsPrint]    - stdout handler for pyjs
 * @param {Function} [options.pyjsError]    - stderr handler for pyjs
 *
 * @returns {Promise<{pyjsModule: object, cxWasm: object, result: object, plan: object}>}
 */
export async function bootstrapConda({
    lockfileUrl,
    platform = 'emscripten-wasm32',
    pythonVersion = [3, 13],
    prefix = '',
    channels = ['https://repo.prefix.dev/emscripten-forge-4x', 'conda-forge'],
    solver = 'emscripten',
    packageBaseUrl,
    onProgress,
    onLog,
    pyjsPrint,
    pyjsError,
}) {
    const log = (msg, level = 'info') => onLog?.(msg, level);

    // ── Step 1: Load cx-wasm WASM module ───────────────────────────────────
    log('Loading cx-wasm module...');
    await init();
    const cxVersion = cx_init();
    const solveVersion = cx_solve_init();
    log(`cx-wasm loaded: ${cxVersion} / ${solveVersion}`, 'ok');

    // Expose globals that Python code invokes via js.*
    window.cx_extract_package = cx_extract_package;
    window.cx_solve = cx_solve;

    const cxWasm = {
        cx_init, cx_bootstrap_streaming, cx_bootstrap_plan,
        cx_extract_package, cx_solve_init, cx_solve
    };

    // ── Step 2: Create pyjs Module ─────────────────────────────────────────
    log('Initializing pyjs runtime...');
    const pyjsModule = await createModule({
        print: pyjsPrint || ((text) => log(text)),
        error: pyjsError || ((text) => log(text, 'warn')),
    });
    log('pyjs Module created (MEMFS ready)', 'ok');

    const FS = pyjsModule.FS;

    // ── Step 3: Fetch and rewrite lockfile ─────────────────────────────────
    log('Fetching lockfile...');
    const resp = await fetch(lockfileUrl);
    if (!resp.ok) throw new Error('Failed to fetch lockfile: HTTP ' + resp.status);
    const rawLockfile = await resp.text();
    const basePkgUrl = packageBaseUrl || (window.location.origin + '/packages');
    const lockfileText = rewriteLockfileUrls(rawLockfile, basePkgUrl);
    log('Lockfile loaded and URLs rewritten', 'ok');

    // ── Step 4: Plan and stream packages into MEMFS ────────────────────────
    const plan = cx_bootstrap_plan(lockfileText, platform);
    log(`Plan: ${plan.package_count} packages, ${formatSize(plan.total_download_size)} to download`);

    let fileCount = 0;
    let totalBytes = 0;
    const sharedLibs = [];
    const pkgIndexData = new Map();

    const onFile = (pkgName, path, bytes) => {
        if (path === 'info/index.json') {
            try {
                pkgIndexData.set(pkgName, JSON.parse(new TextDecoder().decode(bytes)));
            } catch (_) { /* ignore */ }
        }
        if (path.startsWith('info/')) return;

        fileCount++;
        totalBytes += bytes.length;
        const dest = prefix + '/' + path;
        ensureDir(FS, dest.substring(0, dest.lastIndexOf('/')));
        FS.writeFile(dest, bytes);

        if (path.endsWith('.so')) {
            sharedLibs.push(prefix + '/' + path);
        }

        onProgress?.(fileCount, plan.package_count, pkgName);
    };

    const streamProgress = (current, total, name) => {
        onProgress?.(current, total, name);
    };

    const result = await cx_bootstrap_streaming(lockfileText, platform, streamProgress, onFile);

    if (result.errors.length > 0) {
        log(`Completed with ${result.errors.length} error(s)`, 'warn');
        for (const err of result.errors) log('  ' + err, 'error');
    } else {
        log(`Bootstrap: ${result.packages_installed} packages, ${fileCount} files, ${formatSize(totalBytes)}`, 'ok');
    }

    // ── Step 5: Write conda-meta records ───────────────────────────────────
    const planByName = new Map();
    for (const p of plan.packages) {
        planByName.set(p.name, p);
    }
    const metaDir = prefix + '/conda-meta';
    ensureDir(FS, metaDir);
    let metaCount = 0;
    for (const [pkgName, idx] of pkgIndexData) {
        const name = idx.name || pkgName;
        const version = idx.version || '0';
        const build = idx.build || 'unknown';
        const filename = `${name}-${version}-${build}.json`;
        const pm = planByName.get(name);
        if (pm) {
            idx.url = pm.url;
            idx.channel = pm.channel;
            idx.fn = pm.fn_name;
            if (pm.sha256) idx.sha256 = pm.sha256;
            if (pm.md5) idx.md5 = pm.md5;
            if (pm.size) idx.size = pm.size;
        }
        FS.writeFile(metaDir + '/' + filename, new TextEncoder().encode(JSON.stringify(idx, null, 2)));
        metaCount++;
    }
    log(`Wrote ${metaCount} conda-meta records`, 'ok');

    // ── Step 6: Initialize Python interpreter ──────────────────────────────
    log('Initializing Python interpreter...');
    const pyPrefix = prefix || '/';
    await pyjsModule.init_phase_1(pyPrefix, pythonVersion, true);
    log('Python interpreter started (init_phase_1)', 'ok');
    pyjsModule.init_phase_2(pyPrefix, pythonVersion, true);
    log('pyjs bridge ready (init_phase_2)', 'ok');

    // ── Step 7: Load shared libraries ──────────────────────────────────────
    if (sharedLibs.length > 0) {
        log(`Loading ${sharedLibs.length} shared libraries...`);
        for (const soPath of sharedLibs) {
            try {
                pyjsModule.loadDynamicLibrary(soPath, { global: true, nodelete: true });
            } catch (e) {
                log(`Warning: failed to load ${soPath}: ${e}`, 'warn');
            }
        }
        log('Shared libraries loaded', 'ok');
    }

    // ── Step 8: Configure conda environment ────────────────────────────────
    const channelsYaml = channels.map(c => `  - ${c}`).join('\\n');
    pyjsModule['exec'](`
import sys, os, glob

PREFIX = "${pyPrefix}"

sp_paths = ["${prefix}/site-packages"]
for p in glob.glob(PREFIX + "lib/python*/site-packages"):
    sp_paths.append(p)

for sp in sp_paths:
    if sp not in sys.path and os.path.isdir(sp):
        sys.path.insert(0, sp)

os.environ["HOME"] = "/home/web_user"
os.environ["CONDA_ROOT"] = PREFIX
os.environ["CONDA_PREFIX"] = PREFIX
os.environ["CONDA_DEFAULT_ENV"] = "base"
os.environ["CONDA_SOLVER"] = "${solver}"
os.environ["CONDA_SUBDIR"] = "${platform}"

os.makedirs("/home/web_user", exist_ok=True)

condarc = PREFIX + ".condarc"
with open(condarc, "w") as f:
    f.write("channels:\\n${channelsYaml}\\n")

history = PREFIX + "conda-meta/history"
os.makedirs(os.path.dirname(history), exist_ok=True)
with open(history, "w") as f:
    f.write("==> conda-express bootstrap <==\\n")
`);
    log('Python environment configured', 'ok');

    // ── Step 9: Patch urllib3 Emscripten fetch backend for pyjs ────────────
    try {
        pyjsModule['exec'](`
import js
import json
from email.parser import Parser

_HEADERS_TO_IGNORE = ("user-agent",)

def _pyjs_send_request(request):
    from urllib3.contrib.emscripten.response import EmscriptenResponse

    headers_dict = {
        k: v for k, v in request.headers.items()
        if k.lower() not in _HEADERS_TO_IGNORE
    }
    headers_json = json.dumps(headers_dict) if headers_dict else None

    body = request.body
    if isinstance(body, bytes):
        body = body.decode("latin-1")

    xhr_obj = js.XMLHttpRequest.new()
    xhr_obj.open(request.method, request.url, False)
    xhr_obj.overrideMimeType("text/plain; charset=x-user-defined")
    for k, v in headers_dict.items():
        xhr_obj.setRequestHeader(k, v)
    xhr_obj.send(body)

    status = int(str(xhr_obj.status))
    raw_headers = str(xhr_obj.getAllResponseHeaders())
    resp_headers = dict(Parser().parsestr(raw_headers))
    resp_text = str(xhr_obj.responseText) if xhr_obj.responseText else ""
    resp_body = resp_text.encode("latin-1")

    return EmscriptenResponse(
        status_code=status,
        headers=resp_headers,
        body=resp_body,
        request=request,
    )

import urllib3.contrib.emscripten.fetch as _ef
import urllib3.contrib.emscripten.connection as _ec
_ef.send_request = _pyjs_send_request
_ec.send_request = _pyjs_send_request
`);
        log('urllib3 Emscripten fetch backend patched for pyjs', 'ok');
    } catch (e) {
        log('HTTP patch warning: ' + formatPyError(e), 'warn');
    }

    // ── Step 10: Verify conda import ───────────────────────────────────────
    try {
        pyjsModule['exec'](`
import conda
print(f"conda version: {conda.__version__}")
`);
        log('conda imported successfully', 'ok');
    } catch (e) {
        log('import conda failed: ' + formatPyError(e), 'warn');
    }

    return { pyjsModule, cxWasm, result, plan };
}

// ─── Internal helpers ──────────────────────────────────────────────────────────

function formatSize(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}
