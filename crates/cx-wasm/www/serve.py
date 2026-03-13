#!/usr/bin/env python3
"""Dev HTTP server that disables all browser caching.

Drop-in replacement for ``python -m http.server`` with no-cache headers.
Usage: python serve.py [port] [--directory DIR]
"""
import argparse
import functools
import http.server


class NoCacheHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")
        super().end_headers()


parser = argparse.ArgumentParser()
parser.add_argument("port", nargs="?", type=int, default=8080)
parser.add_argument("--directory", "-d", default=".")
args = parser.parse_args()

handler = functools.partial(NoCacheHandler, directory=args.directory)
print(f"Serving {args.directory!r} on http://localhost:{args.port} (no-cache)")
http.server.HTTPServer(("", args.port), handler).serve_forever()
