/**
 * cx-worker.js — Web Worker for conda-in-browser.
 *
 * Owns all heavy computation: WASM module, pyjs runtime, package streaming,
 * IndexedDB caching, and Python/conda execution.  The main thread communicates
 * via Comlink RPC; streaming output (logs, progress, stdout/stderr) is sent as
 * side-channel postMessage events tagged with `__cx: true`.
 *
 * Loaded as a classic Worker; uses importScripts for pyjs runtime and dynamic
 * import() for ES modules (Comlink, cx_wasm).
 */
'use strict';

// pyjs runtime — synchronous load, creates `createModule` on global scope
importScripts('./pyjs_runtime_browser.js');

// ─── Sync XHR helpers (no deprecation warning in Workers) ────────────────────

self.sync_fetch_text = function (url) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', url, false);
    xhr.send();
    if (xhr.status >= 200 && xhr.status < 300) return xhr.responseText;
    throw new Error('HTTP ' + xhr.status + ' for ' + url);
};

self.sync_fetch_binary = function (url) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', url, false);
    xhr.responseType = 'arraybuffer';
    xhr.send();
    if (xhr.status >= 200 && xhr.status < 300) {
        return new Uint8Array(xhr.response);
    }
    throw new Error('HTTP ' + xhr.status + ' for ' + url);
};

// ─── Async init: load ES modules and expose Comlink API ──────────────────────

(async () => {
    var _bustCache = '?v=' + Date.now();
    const Comlink = await import('./vendor/comlink.mjs');
    const {
        default: init, cx_init, cx_bootstrap_streaming, cx_bootstrap_plan,
        cx_extract_package, cx_solve_init, cx_fetch_and_solve
    } = await import('./cx_wasm.js' + _bustCache);

    // ── Worker state ─────────────────────────────────────────────────────

    let pyjsModule = null;
    let FS = null;
    let _ready = false;
    let _fromCache = false;

    // ── Side-channel event emitter ───────────────────────────────────────

    function emit(type, data) {
        self.postMessage({ __cx: true, type, ...data });
    }

    function log(msg, level) {
        level = level || 'info';
        var tag = '[cx-bootstrap]';
        if (level === 'error') console.error(tag, msg);
        else if (level === 'warn') console.warn(tag, msg);
        else console.log(tag, msg);
        emit('log', { msg: msg, level: level });
    }

    // ── IndexedDB cache utilities ────────────────────────────────────────

    var CACHE_DB_NAME = 'cx-bootstrap';
    var CACHE_DB_VERSION = 1;
    var CACHE_STORE_PACKAGES = 'packages';
    var CACHE_STORE_META = 'meta';

    function openCacheDB() {
        return new Promise(function (resolve, reject) {
            var req = indexedDB.open(CACHE_DB_NAME, CACHE_DB_VERSION);
            req.onupgradeneeded = function () {
                var db = req.result;
                if (!db.objectStoreNames.contains(CACHE_STORE_PACKAGES)) {
                    db.createObjectStore(CACHE_STORE_PACKAGES, { keyPath: 'name' });
                }
                if (!db.objectStoreNames.contains(CACHE_STORE_META)) {
                    db.createObjectStore(CACHE_STORE_META);
                }
            };
            req.onsuccess = function () { resolve(req.result); };
            req.onerror = function () { reject(req.error); };
        });
    }

    async function hashLockfile(text) {
        var data = new TextEncoder().encode(text);
        var buf = await crypto.subtle.digest('SHA-256', data);
        return Array.from(new Uint8Array(buf)).map(function (b) {
            return b.toString(16).padStart(2, '0');
        }).join('');
    }

    function getCachedBootstrap(db, hash) {
        return new Promise(function (resolve, reject) {
            var tx = db.transaction([CACHE_STORE_META, CACHE_STORE_PACKAGES], 'readonly');
            var metaStore = tx.objectStore(CACHE_STORE_META);
            var getHash = metaStore.get('lockfileHash');
            getHash.onsuccess = function () {
                if (getHash.result !== hash) { resolve(null); return; }
                var pkgStore = tx.objectStore(CACHE_STORE_PACKAGES);
                var getAll = pkgStore.getAll();
                getAll.onsuccess = function () { resolve(getAll.result); };
                getAll.onerror = function () { reject(getAll.error); };
            };
            getHash.onerror = function () { reject(getHash.error); };
        });
    }

    function saveCachedBootstrap(db, hash, packages) {
        return new Promise(function (resolve, reject) {
            var tx = db.transaction([CACHE_STORE_META, CACHE_STORE_PACKAGES], 'readwrite');
            var metaStore = tx.objectStore(CACHE_STORE_META);
            metaStore.put(hash, 'lockfileHash');
            metaStore.put(Date.now(), 'timestamp');
            metaStore.put(CACHE_DB_VERSION, 'version');
            var pkgStore = tx.objectStore(CACHE_STORE_PACKAGES);
            pkgStore.clear();
            for (var i = 0; i < packages.length; i++) pkgStore.put(packages[i]);
            tx.oncomplete = function () { resolve(); };
            tx.onerror = function () { reject(tx.error); };
        });
    }

    function clearBootstrapCacheDB(db) {
        if (db) {
            return new Promise(function (resolve, reject) {
                var tx = db.transaction([CACHE_STORE_META, CACHE_STORE_PACKAGES], 'readwrite');
                tx.objectStore(CACHE_STORE_META).clear();
                tx.objectStore(CACHE_STORE_PACKAGES).clear();
                tx.oncomplete = function () { resolve(); };
                tx.onerror = function () { reject(tx.error); };
            });
        }
        return new Promise(function (resolve, reject) {
            var req = indexedDB.deleteDatabase(CACHE_DB_NAME);
            req.onsuccess = function () { resolve(); };
            req.onerror = function () { reject(req.error); };
        });
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    function ensureDir(fs, path) {
        var parts = path.split('/').filter(Boolean);
        var cur = '';
        for (var i = 0; i < parts.length; i++) {
            cur += '/' + parts[i];
            try { fs.mkdir(cur); } catch (_) { /* exists */ }
        }
    }

    function rewriteLockfileUrls(text, packageBaseUrl) {
        var result = text;
        var match = result.match(/url:\s*file:\/\/\/(\S+)/);
        if (match) {
            var captured = match[1].replace(/\/$/, '');
            var absPath = '/' + captured;
            var base = packageBaseUrl.replace(/\/$/, '');
            result = result.replaceAll('file://' + absPath, base);
            result = result.replaceAll(absPath + '/noarch/', base + '/');
        }
        return result;
    }

    function formatSize(bytes) {
        if (bytes < 1024) return bytes + ' B';
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
        return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    }

    function formatPyError(e) {
        if (e instanceof Error) return e.message;
        if (typeof e === 'string') return e;
        try {
            if (e && typeof e._getattr === 'function') return e._getattr('__str__')();
        } catch (_) {}
        try {
            var s = e.toString();
            if (s !== '[object Object]') return s;
        } catch (_) {}
        try { return JSON.stringify(e); } catch (_) {}
        return String(e);
    }

    // ── Bootstrap sequence ───────────────────────────────────────────────

    async function doBootstrap(opts) {
        var lockfileUrl = opts.lockfileUrl;
        var platform = opts.platform || 'emscripten-wasm32';
        var pythonVersion = opts.pythonVersion || [3, 13];
        var prefix = opts.prefix || '';
        var channels = opts.channels || [
            'https://repo.prefix.dev/emscripten-forge-4x',
            'https://conda.anaconda.org/conda-forge'
        ];
        var solver = opts.solver || 'emscripten';
        var packageBaseUrl = opts.packageBaseUrl;
        var useCache = opts.useCache !== false;
        var forceRefresh = opts.forceRefresh === true;

        // Step 1: Load cx-wasm
        log('Loading cx-wasm module...');
        await init();
        var cxVersion = cx_init();
        var solveVersion = cx_solve_init();
        log('cx-wasm loaded: ' + cxVersion + ' / ' + solveVersion, 'ok');

        self.cx_extract_package = cx_extract_package;

        // Combined fetch + solve: fetches repodata for all channels/subdirs,
        // decodes directly into solver records (no JSON roundtrip), and solves.
        // Accepts a JSON string, a plain JS object, or a pyjs dict proxy.
        // pyjs dict proxies use getattr for property access which breaks
        // serde_wasm_bindgen, so they are converted via Python's json.dumps
        // (registered as self._cx_json_dumps during bootstrap).
        self.fetch_and_solve = function (request) {
            var jsRequest;
            if (typeof request === 'string') {
                jsRequest = JSON.parse(request);
            } else if (self._cx_json_dumps && typeof request === 'object' && request !== null) {
                var jsonStr = String(self._cx_json_dumps(request));
                jsRequest = JSON.parse(jsonStr);
            } else {
                jsRequest = request;
            }
            return cx_fetch_and_solve(
                jsRequest, self.sync_fetch_binary, self.sync_fetch_text);
        };

        // Step 2: Create pyjs module
        log('Initializing pyjs runtime...');
        pyjsModule = await createModule({
            print: function (text) {
                if (text === 'seek') return;
                emit('print', { text: text });
            },
            error: function (text) {
                if (text === 'seek') return;
                emit('error', { text: text });
            },
        });
        FS = pyjsModule.FS;
        log('pyjs Module created (MEMFS ready)', 'ok');

        // Step 3: Fetch and rewrite lockfile
        log('Fetching lockfile...');
        var resp = await fetch(lockfileUrl);
        if (!resp.ok) throw new Error('Failed to fetch lockfile: HTTP ' + resp.status);
        var rawLockfile = await resp.text();
        var basePkgUrl = packageBaseUrl || (self.location.origin + '/packages');
        var lockfileText = rewriteLockfileUrls(rawLockfile, basePkgUrl);
        log('Lockfile loaded and URLs rewritten', 'ok');

        // Step 4: Plan packages
        var plan = cx_bootstrap_plan(lockfileText, platform);
        log('Plan: ' + plan.package_count + ' packages, ' + formatSize(plan.total_download_size) + ' to download');

        // Step 4.5: Check IndexedDB cache
        var cacheDB = null;
        var lockfileHash = null;
        var cachedPackages = null;
        _fromCache = false;

        if (forceRefresh) {
            try {
                var tmpDB = await openCacheDB();
                await clearBootstrapCacheDB(tmpDB);
                log('Cleared IndexedDB bootstrap cache (forceRefresh)');
            } catch (_) {}
        }

        if (useCache && !forceRefresh) {
            try {
                cacheDB = await openCacheDB();
                lockfileHash = await hashLockfile(lockfileText);
                cachedPackages = await getCachedBootstrap(cacheDB, lockfileHash);
            } catch (e) {
                log('Cache check failed, proceeding without cache: ' + e, 'warn');
            }
        }

        var fileCount = 0;
        var totalBytes = 0;
        var sharedLibs = [];
        var result;

        if (cachedPackages && cachedPackages.length > 0) {
            // Cache HIT
            _fromCache = true;
            log('Restoring ' + cachedPackages.length + ' packages from cache...');

            var metaDir = prefix + '/conda-meta';
            ensureDir(FS, metaDir);

            for (var i = 0; i < cachedPackages.length; i++) {
                var pkg = cachedPackages[i];
                emit('progress', { current: i + 1, total: cachedPackages.length, name: pkg.name });

                for (var j = 0; j < pkg.files.length; j++) {
                    var file = pkg.files[j];
                    var dest = prefix + '/' + file.path;
                    ensureDir(FS, dest.substring(0, dest.lastIndexOf('/')));
                    FS.writeFile(dest, file.data);
                    fileCount++;
                    totalBytes += file.data.length;
                    if (file.path.endsWith('.so')) sharedLibs.push(dest);
                }

                if (pkg.condaMetaFilename && pkg.condaMetaContent) {
                    FS.writeFile(
                        metaDir + '/' + pkg.condaMetaFilename,
                        new TextEncoder().encode(pkg.condaMetaContent)
                    );
                }
            }

            result = {
                packages_installed: cachedPackages.length,
                total_packages: cachedPackages.length,
                errors: [],
            };
            log('Restored from cache: ' + cachedPackages.length + ' packages, ' + fileCount + ' files, ' + formatSize(totalBytes), 'ok');
        } else {
            // Cache MISS
            if (cachedPackages !== null) log('Cache empty or stale, downloading fresh...');

            var pkgIndexData = new Map();
            var cacheBuffer = useCache ? new Map() : null;

            var onFile = function (pkgName, path, bytes) {
                try {
                    if (path === 'info/index.json') {
                        try {
                            pkgIndexData.set(pkgName, JSON.parse(new TextDecoder().decode(bytes)));
                        } catch (_) { /* ignore */ }
                    }
                    if (path.startsWith('info/')) return;

                    fileCount++;
                    totalBytes += bytes.length;
                    var dest = prefix + '/' + path;
                    ensureDir(FS, dest.substring(0, dest.lastIndexOf('/')));
                    FS.writeFile(dest, bytes);

                    if (path.endsWith('.so')) sharedLibs.push(dest);

                    if (cacheBuffer) {
                        if (!cacheBuffer.has(pkgName)) cacheBuffer.set(pkgName, { files: [] });
                        cacheBuffer.get(pkgName).files.push({ path: path, data: new Uint8Array(bytes) });
                    }
                } catch (e) {
                    console.error('[cx-worker] onFile error:', pkgName, path, e);
                    throw e;
                }
            };

            var streamProgress = function (current, total, name) {
                emit('progress', { current: current, total: total, name: name });
            };

            result = await cx_bootstrap_streaming(lockfileText, platform, streamProgress, onFile);

            if (result.errors.length > 0) {
                log('Completed with ' + result.errors.length + ' error(s)', 'warn');
                for (var ei = 0; ei < result.errors.length; ei++) log('  ' + result.errors[ei], 'error');
            } else {
                log('Bootstrap: ' + result.packages_installed + ' packages, ' + fileCount + ' files, ' + formatSize(totalBytes), 'ok');
            }

            // Write conda-meta records
            var planByName = new Map();
            for (var pi = 0; pi < plan.packages.length; pi++) {
                planByName.set(plan.packages[pi].name, plan.packages[pi]);
            }
            var metaDirPath = prefix + '/conda-meta';
            ensureDir(FS, metaDirPath);
            var metaCount = 0;
            pkgIndexData.forEach(function (idx, pkgName) {
                var name = idx.name || pkgName;
                var version = idx.version || '0';
                var build = idx.build || 'unknown';
                var filename = name + '-' + version + '-' + build + '.json';
                var pm = planByName.get(name);
                if (pm) {
                    idx.url = pm.url;
                    idx.channel = pm.channel;
                    idx.fn = pm.fn_name;
                    if (pm.sha256) idx.sha256 = pm.sha256;
                    if (pm.md5) idx.md5 = pm.md5;
                    if (pm.size) idx.size = pm.size;
                }
                var metaJson = JSON.stringify(idx, null, 2);
                FS.writeFile(metaDirPath + '/' + filename, new TextEncoder().encode(metaJson));
                metaCount++;

                if (cacheBuffer && cacheBuffer.has(pkgName)) {
                    var entry = cacheBuffer.get(pkgName);
                    entry.condaMetaFilename = filename;
                    entry.condaMetaContent = metaJson;
                }
            });
            log('Wrote ' + metaCount + ' conda-meta records', 'ok');

            // Save to IndexedDB cache
            if (useCache && result.errors.length === 0) {
                try {
                    if (!cacheDB) cacheDB = await openCacheDB();
                    if (!lockfileHash) lockfileHash = await hashLockfile(lockfileText);
                    var packages = [];
                    cacheBuffer.forEach(function (entry, name) {
                        packages.push({
                            name: name,
                            files: entry.files,
                            condaMetaFilename: entry.condaMetaFilename || null,
                            condaMetaContent: entry.condaMetaContent || null,
                        });
                    });
                    await saveCachedBootstrap(cacheDB, lockfileHash, packages);
                    log('Cached ' + packages.length + ' packages to IndexedDB', 'ok');
                } catch (e) {
                    log('Failed to save cache: ' + e, 'warn');
                }
            }
        }

        // Step 6: Initialize Python interpreter
        log('Initializing Python interpreter...');
        var pyPrefix = prefix || '/';
        try {
            await pyjsModule.init_phase_1(pyPrefix, pythonVersion, true);
            log('Python interpreter started (init_phase_1)', 'ok');
        } catch (e) {
            log('init_phase_1 failed: ' + e, 'error');
            throw e;
        }
        try {
            pyjsModule.init_phase_2(pyPrefix, pythonVersion, true);
            log('pyjs bridge ready (init_phase_2)', 'ok');
        } catch (e) {
            log('init_phase_2 failed: ' + e, 'error');
            throw e;
        }

        // Step 7: Load shared libraries
        if (sharedLibs.length > 0) {
            log('Loading ' + sharedLibs.length + ' shared libraries...');
            for (var si = 0; si < sharedLibs.length; si++) {
                try {
                    pyjsModule.loadDynamicLibrary(sharedLibs[si], { global: true, nodelete: true });
                } catch (e) {
                    log('Warning: failed to load ' + sharedLibs[si] + ': ' + e, 'warn');
                }
            }
            log('Shared libraries loaded', 'ok');
        }

        // Step 8: Configure conda environment
        log('Configuring conda environment...');
        try {
            var channelsYaml = channels.map(function (c) { return '  - ' + c; }).join('\\n');
            pyjsModule['exec'](
                'import sys, os, glob\n' +
                '\n' +
                'PREFIX = "' + pyPrefix + '"\n' +
                '\n' +
                'sp_paths = ["' + prefix + '/site-packages"]\n' +
                'for p in glob.glob(PREFIX + "lib/python*/site-packages"):\n' +
                '    sp_paths.append(p)\n' +
                '\n' +
                'for sp in sp_paths:\n' +
                '    if sp not in sys.path and os.path.isdir(sp):\n' +
                '        sys.path.insert(0, sp)\n' +
                '\n' +
                'os.environ["HOME"] = "/home/web_user"\n' +
                'os.environ["CONDA_ROOT"] = PREFIX\n' +
                'os.environ["CONDA_PREFIX"] = PREFIX\n' +
                'os.environ["CONDA_DEFAULT_ENV"] = "base"\n' +
                'os.environ["CONDA_SOLVER"] = "' + solver + '"\n' +
                'os.environ["CONDA_SUBDIR"] = "' + platform + '"\n' +
                'os.environ["CONDA_NUMBER_CHANNEL_NOTICES"] = "0"\n' +
                '\n' +
                'os.makedirs("/home/web_user", exist_ok=True)\n' +
                '\n' +
                'condarc = PREFIX + ".condarc"\n' +
                'with open(condarc, "w") as f:\n' +
                '    f.write("channels:\\n' + channelsYaml + '\\n")\n' +
                '\n' +
                'history = PREFIX + "conda-meta/history"\n' +
                'os.makedirs(os.path.dirname(history), exist_ok=True)\n' +
                'with open(history, "w") as f:\n' +
                '    f.write("==> conda-express bootstrap <==\\n")\n'
            );
            log('Python environment configured', 'ok');
        } catch (e) {
            log('Python environment config failed: ' + formatPyError(e), 'error');
            throw e;
        }

        // Step 8.5: Register json.dumps helper for pyjs dict → JS object conversion.
        // When Python passes a dict to JS, pyjs wraps it as a Proxy that uses
        // getattr for property access, which breaks serde_wasm_bindgen.  This
        // helper lets the JS wrapper convert dicts to JSON strings via Python.
        try {
            pyjsModule['exec'](
                'import json, js\n' +
                'js._cx_json_dumps = json.dumps\n'
            );
        } catch (e) {
            log('json.dumps helper registration failed: ' + formatPyError(e), 'warn');
        }

        // Step 9: Patch urllib3 Emscripten fetch backend for pyjs.
        // Uses sync XHR with responseType='arraybuffer' (supported in Workers)
        // to avoid slow character-by-character text→bytes conversion.
        try {
            pyjsModule['exec'](
                'import js\n' +
                'from email.parser import Parser\n' +
                '\n' +
                '_HEADERS_TO_IGNORE = ("user-agent",)\n' +
                '\n' +
                'def _pyjs_send_request(request):\n' +
                '    import pyjs\n' +
                '    from urllib3.contrib.emscripten.response import EmscriptenResponse\n' +
                '\n' +
                '    headers_dict = {\n' +
                '        k: v for k, v in request.headers.items()\n' +
                '        if k.lower() not in _HEADERS_TO_IGNORE\n' +
                '    }\n' +
                '\n' +
                '    body = request.body\n' +
                '    if isinstance(body, bytes):\n' +
                '        body = body.decode("latin-1")\n' +
                '\n' +
                '    xhr_obj = js.XMLHttpRequest.new()\n' +
                '    xhr_obj.open(request.method, request.url, False)\n' +
                '    xhr_obj.responseType = "arraybuffer"\n' +
                '    for k, v in headers_dict.items():\n' +
                '        xhr_obj.setRequestHeader(k, v)\n' +
                '    xhr_obj.send(body)\n' +
                '\n' +
                '    status = int(str(xhr_obj.status))\n' +
                '    raw_headers = str(xhr_obj.getAllResponseHeaders())\n' +
                '    resp_headers = dict(Parser().parsestr(raw_headers))\n' +
                '    resp_body = bytes(pyjs.to_py(js.Uint8Array.new(xhr_obj.response)))\n' +
                '\n' +
                '    return EmscriptenResponse(\n' +
                '        status_code=status,\n' +
                '        headers=resp_headers,\n' +
                '        body=resp_body,\n' +
                '        request=request,\n' +
                '    )\n' +
                '\n' +
                'import urllib3.contrib.emscripten.fetch as _ef\n' +
                'import urllib3.contrib.emscripten.connection as _ec\n' +
                '_ef.send_request = _pyjs_send_request\n' +
                '_ec.send_request = _pyjs_send_request\n'
            );
            log('urllib3 Emscripten fetch backend patched for pyjs', 'ok');
        } catch (e) {
            log('HTTP patch warning: ' + formatPyError(e), 'warn');
        }

        // Step 10: Patch Emscripten-incompatible conda internals
        try {
            pyjsModule['exec'](
                'import sys\n' +
                'if sys.platform == "emscripten":\n' +
                '    import fcntl\n' +
                '    if not hasattr(fcntl, "lockf"):\n' +
                '        fcntl.lockf = lambda fd, op, *a, **kw: None\n' +
                '    if not hasattr(fcntl, "flock"):\n' +
                '        fcntl.flock = lambda fd, op: None\n' +
                '\n' +
                '    from conda.core import solve as _solve\n' +
                '    _solve.Solver._notify_conda_outdated = lambda self, link_precs: None\n' +
                '\n' +
                '    from conda.gateways.repodata import RepodataCache\n' +
                '    _orig_save = RepodataCache.save\n' +
                '    def _safe_save(self, raw_repodata):\n' +
                '        try:\n' +
                '            return _orig_save(self, raw_repodata)\n' +
                '        except (AttributeError, OSError):\n' +
                '            pass\n' +
                '    RepodataCache.save = _safe_save\n'
            );
            log('Emscripten conda patches applied', 'ok');
        } catch (e) {
            log('Emscripten conda patches warning: ' + formatPyError(e), 'warn');
        }

        // Step 11: Verify conda import
        try {
            pyjsModule['exec'](
                'import conda\n' +
                'print(f"conda version: {conda.__version__}")\n'
            );
            log('conda imported successfully', 'ok');
        } catch (e) {
            log('import conda failed: ' + formatPyError(e), 'warn');
        }

        _ready = true;

        return {
            packages_installed: result.packages_installed,
            total_packages: result.total_packages,
            errors: result.errors,
            fromCache: _fromCache,
            plan: {
                package_count: plan.package_count,
                total_download_size: plan.total_download_size,
            },
        };
    }

    // ── Comlink API ──────────────────────────────────────────────────────

    Comlink.expose({
        bootstrap: doBootstrap,

        runPythonSync: function (code) {
            if (!pyjsModule) throw new Error('Not bootstrapped yet');
            pyjsModule['exec'](code);
        },

        runPython: async function (code) {
            if (!pyjsModule) throw new Error('Not bootstrapped yet');
            return await pyjsModule['async_exec_' + 'eval'](code);
        },

        clearCache: async function () {
            try {
                var db = await openCacheDB();
                await clearBootstrapCacheDB(db);
            } catch (_) {
                await clearBootstrapCacheDB(null);
            }
        },

        getState: function () {
            return { ready: _ready, fromCache: _fromCache };
        },
    });

    // Signal to main thread that Comlink API is ready
    self.postMessage({ __cx: true, type: 'ready' });
})();
