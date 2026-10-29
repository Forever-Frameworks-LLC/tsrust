from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from check_links import check


class Links(unittest.TestCase):
    def test_base_relative_assets_and_anchors(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'style.css').write_text('')
            (root / 'guide').mkdir()
            (root / 'guide/index.html').write_text('<h1 id="hello">Hello</h1><a href="../">Home</a>')
            prefix = '<link rel="canonical" href="https://example.com/docs/"><link href="/docs/style.css">'
            (root / 'index.html').write_text(prefix + '<a href="guide/#hello">Guide</a>')
            (root / '404.html').write_text('<link rel="canonical" href="https://example.com/docs/404/"><h1 id="top">Missing</h1><a href="/docs/404/#top">Top</a>')
            redirect = '<meta http-equiv="refresh" content="0;url=/docs/guide/#hello"><link rel="canonical" href="https://example.com/docs/guide/#hello"><a href="/docs/guide/#hello">Moved</a>'
            (root / 'old').mkdir()
            (root / 'old/index.html').write_text(redirect)
            self.assertEqual(check(root), 4)
            (root / 'old/index.html').write_text(redirect.replace('#hello', '#absent'))
            with self.assertRaises(ValueError):
                check(root)
            (root / 'old/index.html').write_text(redirect)
            for broken in ('guide/#absent', 'missing/', '/outside/', '%2e%2e/private'):
                with self.subTest(link=broken):
                    (root / 'index.html').write_text(prefix + f'<a href="{broken}">Broken</a>')
                    with self.assertRaises(ValueError):
                        check(root)


if __name__ == '__main__':
    unittest.main()
