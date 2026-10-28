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
            self.assertEqual(check(root), 3)
            for broken in ('guide/#absent', 'missing/', '/outside/', '%2e%2e/private'):
                with self.subTest(link=broken):
                    (root / 'index.html').write_text(prefix + f'<a href="{broken}">Broken</a>')
                    with self.assertRaises(ValueError):
                        check(root)


if __name__ == '__main__':
    unittest.main()
