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


class RustControlTests(unittest.TestCase):
    def test_build_phase_parser_rejects_cache_reuse(self):
        import rust_control
        phases = rust_control.build_timings(
            '[build] check=1.2ms emit=0.7ms backend=29.4ms cache=miss (0.1ms) native+link=160.3ms')
        self.assertEqual(phases['checkMs'], 1.2)
        self.assertEqual(phases['nativeBuildMs'], 160.3)
        with self.assertRaises(ValueError):
            rust_control.build_timings(
                '[build] check=1.2ms emit=0.7ms backend=29.4ms cache=hit (0.1ms) native+link=0.0ms')
        with self.assertRaises(ValueError):
            rust_control.build_timings('missing build timings')

if __name__ == '__main__':
    unittest.main()
