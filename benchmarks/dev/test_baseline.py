import importlib.util
import pathlib
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('baseline', pathlib.Path(__file__).with_name('baseline.py'))
baseline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(baseline)

class BaselineTests(unittest.TestCase):
    def test_generated_graph_and_edit_oracle(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            baseline.project(root, 4)
            self.assertEqual(len(list(root.glob('*.tsr'))), 6)
            self.assertEqual(baseline.expected(4, 1, 0), 10)
            self.assertEqual(baseline.expected(4, 1, 10), 20)
            self.assertEqual(baseline.expected(4, 2, 10), 24)
            self.assertIn("./shared.tsr", (root / 'module3.tsr').read_text())
            self.assertEqual((root / 'main.tsr').read_text().count('import '), 4)
            self.assertEqual((root / 'main.tsr').read_text().count('total +='), 4)

    def test_cpu_time_conversion(self):
        self.assertEqual(baseline.cpu_seconds('1:02.50'), 62.5)
        self.assertEqual(baseline.cpu_seconds('02:01:02'), 7262)

if __name__ == '__main__':
    unittest.main()
