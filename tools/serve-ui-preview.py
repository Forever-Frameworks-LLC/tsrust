"""Serve a built UI with history fallback for document navigation, never missing assets."""
import argparse
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit


class PreviewHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        path = urlsplit(self.path).path
        target = Path(self.translate_path(path))
        if not target.exists() and 'text/html' in self.headers.get('Accept', ''):
            self.path = '/index.html'
        super().do_GET()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--port', type=int, default=4317)
    args = parser.parse_args()
    root = args.directory.resolve(strict=True)
    if not (root / 'index.html').is_file():
        parser.error('directory needs a built index.html')
    handler = lambda *a, **kw: PreviewHandler(*a, directory=str(root), **kw)
    with ThreadingHTTPServer(('127.0.0.1', args.port), handler) as server:
        print(f'UI preview: http://127.0.0.1:{args.port}/', flush=True)
        server.serve_forever()


if __name__ == '__main__':
    main()
