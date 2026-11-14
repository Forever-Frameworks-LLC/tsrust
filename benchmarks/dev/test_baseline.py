import importlib.util
import pathlib
import tempfile
import unittest
from unittest import mock

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

    def test_busy_machine_prevents_dev_start(self):
        with mock.patch.object(baseline, 'wait_for_quiet_machine', side_effect=RuntimeError('busy')):
            with mock.patch.object(baseline, 'Dev') as dev:
                with self.assertRaisesRegex(RuntimeError, 'busy'):
                    baseline.sample('unused-compiler', 4, 3)
                dev.assert_not_called()

    def test_completed_dev_samples_export_to_the_shared_measurement_format(self):
        metrics = ['coldStartMs', 'warmStartMs', 'leafEditMs', 'sharedEditMs', 'configEditMs']
        rows = [{'modules': 6, 'observations': dict.fromkeys(metrics, value)} for value in [10, 12, 11]]
        manifest, execution = baseline.measurement_records(rows)
        self.assertEqual(len(manifest['workloads']), 5)
        self.assertTrue(all(row['iterations'] == 1 for row in manifest['workloads']))
        self.assertEqual({row['engine'] for row in execution}, {'tsrust-dev'})
        cold = [row['durations'][0] for row in execution if row['workload'] == 'dev-loop-6-modules-coldStartMs']
        self.assertEqual(cold, [10, 12, 11])
        self.assertEqual(baseline.measurement_records([]), ({'workloads': []}, []))

    def test_cpu_time_conversion(self):
        self.assertEqual(baseline.cpu_seconds('1:02.50'), 62.5)
        self.assertEqual(baseline.cpu_seconds('02:01:02'), 7262)


class RustControlTests(unittest.TestCase):
    def test_busy_machine_prevents_timed_process(self):
        import rust_control
        with mock.patch.object(rust_control.baseline, 'wait_for_quiet_machine', side_effect=RuntimeError('busy')):
            with mock.patch.object(rust_control.subprocess, 'run') as process:
                with self.assertRaisesRegex(RuntimeError, 'busy'):
                    rust_control.run(['unused-compiler'], pathlib.Path.cwd(), {})
                process.assert_not_called()

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
