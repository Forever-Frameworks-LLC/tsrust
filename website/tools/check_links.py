"""Check rendered local links, anchors and assets using the built canonical URL."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urljoin, urlparse
import sys


class Page(HTMLParser):
    def __init__(self, text):
        super().__init__()
        self.links, self.ids, self.canonical = [], set(), None
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if 'id' in attrs:
            self.ids.add(attrs['id'])
        if tag == 'link' and attrs.get('rel') == 'canonical':
            self.canonical = attrs.get('href')
            return
        for name in ('href', 'src'):
            if name in attrs:
                self.links.append(attrs[name])


def check(root):
    root = root.resolve()
    pages = {path: Page(path.read_text()) for path in root.rglob('*.html')}
    base = pages[root / 'index.html'].canonical
    if not base or not base.endswith('/'):
        raise ValueError('Root page needs an absolute trailing-slash canonical URL')
    origin = urlparse(base)
    for file, page in pages.items():
        relative = file.relative_to(root).as_posix()
        current = urljoin(base, relative.removesuffix('index.html'))
        for link in page.links:
            url = urlparse(urljoin(current, link))
            if url.scheme not in ('https', 'http') or url.netloc != origin.netloc:
                continue
            if not url.path.startswith(origin.path):
                raise ValueError(f'{file}: link escapes site base: {link}')
            target = (root / unquote(url.path[len(origin.path):])).resolve()
            if not target.is_relative_to(root):
                raise ValueError(f'{file}: link escapes output: {link}')
            if target.is_dir():
                target /= 'index.html'
            # A page without its own route (404.html) may link to its canonical
            # URL. Redirect pages are canonical to an existing target instead.
            if not target.is_file() and page.canonical and url.path == urlparse(page.canonical).path:
                target = file
            if not target.is_file():
                raise ValueError(f'{file}: missing target: {link}')
            if url.fragment and target in pages and unquote(url.fragment) not in pages[target].ids:
                raise ValueError(f'{file}: missing anchor: {link}')
    return len(pages)


if __name__ == '__main__':
    print(f'Checked links, anchors and assets in {check(Path(sys.argv[1] if len(sys.argv) > 1 else "dist"))} pages.')
