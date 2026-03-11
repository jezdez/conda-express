"""Emscripten conda solver using cx-wasm (resolvo) for browser environments."""

from __future__ import annotations

import json
import logging
import sys
from typing import TYPE_CHECKING

from conda.auxlib import NULL
from conda.base.context import context
from conda.common.io import dashlist
from conda.core.solve import Solver
from conda.models.records import PackageRecord, PrefixRecord

if TYPE_CHECKING:
    from collections.abc import Iterable

    from conda.models.channel import Channel

log = logging.getLogger(__name__)


def _get_js_bridge():
    """Get the cx-wasm bridge from the JS global scope via pyjs."""
    if sys.platform != "emscripten":
        raise RuntimeError(
            "conda-emscripten requires an Emscripten/pyjs environment. "
            "Use CONDA_SOLVER=rattler or CONDA_SOLVER=classic for native environments."
        )
    try:
        import js
        return js
    except ImportError:
        raise RuntimeError(
            "Could not import 'js' module. "
            "conda-emscripten requires pyjs JS bridge."
        )


def _fetch_repodata(
    channel_url: str,
    subdir: str,
    seed_names: list[str] | None = None,
) -> str:
    """Fetch repodata for a channel/subdir, preferring smaller formats.

    Tries in order:
    1. Sharded repodata (CEP-16) via JS helper — fetches only the shards
       for ``seed_names`` and their transitive dependencies.
    2. current_repodata.json (latest versions only, much smaller)
    3. repodata.json (full, can be 50-100+ MB for conda-forge)

    Uses synchronous XMLHttpRequest via the ``js.sync_fetch_text`` helper.
    """
    import js

    base = channel_url.rstrip("/")

    # Try sharded repodata via JS helper (CEP-16)
    if seed_names and hasattr(js, "fetch_sharded_repodata"):
        try:
            import json as _json
            seed_json = _json.dumps(seed_names)
            raw = js.fetch_sharded_repodata(base, subdir, seed_json)
            if raw is None:
                log.debug("Sharded repodata returned None for %s/%s", base, subdir)
            else:
                result = str(raw)
                if result and result not in ("null", "undefined", "None"):
                    log.info(
                        "Fetched sharded repodata for %s/%s (%d seed packages, %d chars)",
                        base, subdir, len(seed_names), len(result),
                    )
                    return result
                log.debug("Sharded repodata empty/null for %s/%s: %r", base, subdir, result[:100])
        except Exception as e:
            log.warning("Sharded repodata failed for %s/%s: %s", base, subdir, e)

    # Try current_repodata.json first (only latest versions, much smaller)
    current_url = f"{base}/{subdir}/current_repodata.json"
    try:
        log.info("Fetching current_repodata from %s", current_url)
        result = str(js.sync_fetch_text(current_url))
        if result:
            log.info("Got current_repodata for %s/%s (%d chars)", base, subdir, len(result))
            return result
    except Exception:
        log.debug("current_repodata.json not available for %s/%s, trying full repodata", base, subdir)

    # Fall back to full repodata.json
    url = f"{base}/{subdir}/repodata.json"
    log.info("Fetching full repodata from %s", url)
    return str(js.sync_fetch_text(url))


def _records_to_json(records: Iterable[PrefixRecord]) -> str:
    """Convert installed PrefixRecord objects to JSON for cx-wasm.

    Rattler's RepoDataRecord requires:
    - ``fn``: a valid archive identifier ending in ``.conda`` or ``.tar.bz2``
    - ``url``: a syntactically valid URL
    """
    result = []
    for rec in records:
        fn = rec.fn or ""
        if not (fn.endswith(".conda") or fn.endswith(".tar.bz2")):
            fn = f"{rec.name}-{rec.version}-{rec.build}.conda"

        channel_str = str(rec.channel) if rec.channel else ""
        if not channel_str or channel_str.startswith("<") or "://" not in channel_str:
            channel = "https://conda.anaconda.org/unknown"
        else:
            channel = channel_str
        subdir = rec.subdir or "noarch"

        url = str(rec.url) if rec.url else ""
        if not url or "://" not in url:
            url = f"{channel}/{subdir}/{fn}"

        entry = {
            "name": rec.name,
            "version": str(rec.version),
            "build": rec.build,
            "build_number": rec.build_number,
            "subdir": subdir,
            "fn": fn,
            "url": url,
            "channel": channel,
            "depends": list(rec.depends or []),
            "constrains": list(rec.constrains or []),
        }
        if rec.md5:
            entry["md5"] = rec.md5
        if rec.sha256:
            entry["sha256"] = rec.sha256
        result.append(entry)
    return json.dumps(result)


def _solution_record_to_package_record(r: dict) -> PackageRecord:
    """Convert a single cx-wasm solution record dict to a conda PackageRecord."""
    channel_url = r.get("channel", "")
    subdir = r.get("subdir", "noarch")

    if channel_url and not channel_url.endswith(("noarch", subdir)):
        channel_with_subdir = f"{channel_url}/{subdir}"
    else:
        channel_with_subdir = channel_url

    kwargs = dict(
        name=r["name"],
        version=str(r["version"]),
        build=r["build"],
        build_number=int(r.get("build_number", 0)),
        channel=channel_with_subdir,
        subdir=subdir,
        fn=r.get("file_name", f"{r['name']}-{r['version']}-{r['build']}.conda"),
        url=r.get("url", ""),
        depends=tuple(r.get("depends", ())),
        constrains=tuple(r.get("constrains", ())),
    )

    kwargs["size"] = int(r.get("size") or 0)
    if r.get("md5"):
        kwargs["md5"] = r["md5"]
    if r.get("sha256"):
        kwargs["sha256"] = r["sha256"]

    return PackageRecord(**kwargs)


def _solution_to_records(solution) -> list[PackageRecord]:
    """Convert cx-wasm solution (JS object or dict) to conda PackageRecords."""
    sol_records = solution["records"] if isinstance(solution, dict) else solution.records
    records = []
    for rec in sol_records:
        if isinstance(rec, dict):
            r = rec
        else:
            r = rec.to_py() if hasattr(rec, "to_py") else dict(rec)
        records.append(_solution_record_to_package_record(r))
    return records


class WasmSolver(Solver):
    """Conda solver implementation that delegates to cx-wasm WASM module.

    Designed for browser/Emscripten environments where the cx-wasm WASM
    module provides dependency resolution via resolvo.

    Selected with CONDA_SOLVER=wasm.
    """

    _uses_ssc = False

    def __init__(
        self,
        prefix: str,
        channels: Iterable[Channel] | None = None,
        subdirs: Iterable[str] = (),
        specs_to_add=(),
        specs_to_remove=(),
        repodata_fn: str = "repodata.json",
        command=NULL,
    ):
        super().__init__(
            prefix,
            channels,
            subdirs,
            specs_to_add,
            specs_to_remove,
            repodata_fn,
            command,
        )
        if not self.subdirs or "noarch" not in self.subdirs:
            self.subdirs = (*self.subdirs, "noarch")

    def solve_final_state(
        self,
        update_modifier=NULL,
        deps_modifier=NULL,
        prune=NULL,
        ignore_pinned=NULL,
        force_remove=NULL,
        should_retry_solve=False,
    ):
        """Solve the environment using cx-wasm WASM module.

        Returns an IndexedSet of PackageRecord in dependency order (roots to
        leaves), consistent with the conda solver plugin contract.
        """
        from boltons.setutils import IndexedSet
        from conda.base.constants import DepsModifier, UpdateModifier
        from conda.core.prefix_data import PrefixData
        from conda.exceptions import PackagesNotFoundError
        from conda.models.prefix_graph import PrefixGraph

        if update_modifier is NULL:
            update_modifier = context.update_modifier
        else:
            update_modifier = UpdateModifier(str(update_modifier).lower())
        if deps_modifier is NULL:
            deps_modifier = context.deps_modifier
        else:
            deps_modifier = DepsModifier(str(deps_modifier).lower())
        if ignore_pinned is NULL:
            ignore_pinned = context.ignore_pinned
        if force_remove is NULL:
            force_remove = context.force_remove
        if prune is NULL:
            prune = False

        prefix_data = PrefixData(self.prefix)
        installed = {rec.name: rec for rec in prefix_data.iter_records()}

        # --- Early exit: force_remove ---
        if self.specs_to_remove and force_remove:
            if self.specs_to_add:
                raise NotImplementedError(
                    "force_remove with specs_to_add is not supported"
                )
            remove_names = {s.name for s in self.specs_to_remove if s.name}
            not_installed = remove_names - set(installed)
            if not_installed:
                raise PackagesNotFoundError(sorted(not_installed))
            remaining = [
                rec for name, rec in installed.items() if name not in remove_names
            ]
            self.neutered_specs = ()
            return IndexedSet(PrefixGraph(remaining).graph)

        # --- Early exit: nothing to do ---
        if not self.specs_to_add and not self.specs_to_remove:
            log.info("WasmSolver: no specs to add or remove, returning current state")
            self.neutered_specs = ()
            return IndexedSet(PrefixGraph(installed.values()).graph)

        # --- Main solve path ---
        js = _get_js_bridge()

        specs = list(self.specs_to_add)
        log.info(
            "WasmSolver: solving with %d specs to add, %d to remove",
            len(self.specs_to_add),
            len(self.specs_to_remove),
        )

        repodata_entries = self._fetch_all_repodata()
        if not repodata_entries:
            raise RuntimeError(
                f"Could not fetch repodata from any channel/subdir combination.\n"
                f"Channels: {dashlist(str(c) for c in self.channels)}\n"
                f"Subdirs: {list(self.subdirs)}"
            )

        installed_json = _records_to_json(installed.values()) if installed else "[]"

        virtual_packages = self._collect_virtual_packages()
        platform = context.subdir or "emscripten-wasm32"

        remove_names = {s.name for s in self.specs_to_remove if s.name}

        solve_specs = [str(s) for s in specs]
        for name in installed:
            if name not in remove_names:
                solve_specs.append(name)

        for i, entry in enumerate(repodata_entries):
            rd = entry["repodata"]
            log.info(
                "repodata_entries[%d] %s/%s: type=%s len=%d first100=%r",
                i, entry["channel"], entry["subdir"],
                type(rd).__name__, len(rd), rd[:100],
            )

        solve_request_json = json.dumps({
            "repodata": repodata_entries,
            "specs": solve_specs,
            "installed": installed_json,
            "virtual_packages": virtual_packages,
            "platform": platform,
        })
        log.info("solve_request_json length: %d", len(solve_request_json))
        solve_request = js.JSON.parse(solve_request_json)

        log.info(
            "WasmSolver: calling cx_solve with %d repodata sources, %d specs",
            len(repodata_entries),
            len(solve_specs),
        )
        solution = js.cx_solve(solve_request)

        solution = json.loads(js.JSON.stringify(solution))

        solved_records = _solution_to_records(solution)
        log.info("WasmSolver: solution has %d packages", len(solved_records))

        # Preserve installed records for unchanged packages so conda
        # doesn't see a channel change and try to reinstall them.
        installed_index = {
            (r.name, str(r.version), r.build): r for r in installed.values()
        }
        records = []
        for rec in solved_records:
            key = (rec.name, str(rec.version), rec.build)
            original = installed_index.get(key)
            records.append(original if original is not None else rec)

        if prune:
            graph = PrefixGraph(records, self.specs_to_add)
            graph.prune()
            records = list(graph.graph)

        self.neutered_specs = ()

        return IndexedSet(PrefixGraph(records).graph)

    def _fetch_all_repodata(self) -> list[dict]:
        """Fetch repodata for all channel/subdir combinations.

        Collects seed package names from specs_to_add and installed packages
        so the sharded repodata fetcher (CEP-16) can do targeted fetching
        instead of downloading the full monolithic repodata.json.
        """
        seed_names = self._collect_seed_names()
        entries = []
        for channel in self.channels:
            channel_url = self._channel_to_url(channel)
            for subdir in self.subdirs:
                try:
                    repodata_json = _fetch_repodata(channel_url, subdir, seed_names)
                    entries.append(
                        {
                            "channel": channel_url,
                            "subdir": subdir,
                            "repodata": repodata_json,
                        }
                    )
                except Exception as e:
                    log.warning(
                        "Failed to fetch repodata for %s/%s: %s",
                        channel_url,
                        subdir,
                        e,
                    )
        return entries

    def _collect_seed_names(self) -> list[str]:
        """Collect package names to seed sharded repodata fetching.

        Includes: specs being added, specs being removed, and all currently
        installed package names (since the solver needs to re-resolve them).
        """
        from conda.core.prefix_data import PrefixData

        names: set[str] = set()
        for s in self.specs_to_add:
            if s.name:
                names.add(s.name)
        for s in self.specs_to_remove:
            if s.name:
                names.add(s.name)

        prefix_data = PrefixData(self.prefix)
        for rec in prefix_data.iter_records():
            names.add(rec.name)

        return sorted(names)

    @staticmethod
    def _channel_to_url(channel: Channel) -> str:
        """Extract a usable URL string from a conda Channel object."""
        if hasattr(channel, "base_url"):
            return str(channel.base_url)
        for url in getattr(channel, "urls", ()):
            return str(url).rsplit("/", 1)[0]
        return str(channel)

    @staticmethod
    def _collect_virtual_packages() -> list[dict]:
        """Collect virtual packages from the plugin manager.

        The conda-emscripten plugin registers ``__unix`` and
        ``__emscripten`` via ``conda_virtual_packages`` hookimpl, so
        they will be included automatically when the subdir is
        ``emscripten-*``.
        """
        vpkgs = []
        for vp in context.plugin_manager.get_virtual_package_records():
            vpkgs.append(
                {
                    "name": vp.name,
                    "version": str(vp.version) if vp.version else "0",
                    "build_string": vp.build or "",
                }
            )
        return vpkgs
