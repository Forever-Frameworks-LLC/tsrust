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

class WorklistContextTests(unittest.TestCase):
    def test_source_guard_covers_additional_roots_and_native_binding_sources(self):
        import worklist_runtime_context as context
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            project = root / 'studio'
            project.mkdir()
            native = project / 'binding.rs'
            native.write_text('fn initial() {}')
            (project / 'target').mkdir()
            (project / 'target/ignored.tsr').write_text('not an input')
            with mock.patch.object(context, 'ROOT', root), mock.patch.object(context, 'CACHE', root / 'cache'), mock.patch.dict('os.environ', {'CARGO_HOME': str(root / 'home')}):
                first = context.source_snapshot([project])
                self.assertIn('studio/binding.rs', first)
                self.assertNotIn('studio/target/ignored.tsr', first)
                native.write_text('fn changed() {}')
                self.assertNotEqual(context.source_snapshot([project]), first)

    def test_context_comparison_accepts_an_initially_empty_cache(self):
        import worklist_runtime_context as context
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            with mock.patch.object(context, 'CACHE', root):
                self.assertEqual(context.keys('runtime/v2'), set())
                entry = root / 'runtime/v2/accepted'
                entry.mkdir(parents=True)
                (entry / 'identity').write_bytes(b'identity')
                (root / 'runtime/v2/incomplete').mkdir()
                self.assertEqual(context.keys('runtime/v2'), {'accepted'})

    def test_context_phases_use_the_public_build_log_without_dev_only_metrics(self):
        import worklist_runtime_context as context
        stderr = '[build] runtime=miss\n[build] check=1ms emit=2ms backend=3ms cache=miss (0.1ms) native+link=4ms'
        self.assertEqual(context.context_build_timings(stderr)['nativeBuildMs'], 4)
        with self.assertRaises(ValueError):
            context.context_build_timings(stderr.replace('runtime=miss', 'runtime=hit'))

    def test_context_guard_tracks_included_cargo_configuration(self):
        import worklist_runtime_context as context
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            config = root / '.cargo'
            config.mkdir()
            (config / 'config.toml').write_text('include = ["extra.toml"]')
            extra = config / 'extra.toml'
            extra.write_text('include = [{path="missing.toml", optional=true}]')
            with mock.patch.object(context, 'ROOT', root), mock.patch.object(context, 'PROJECT', root):
                with mock.patch.object(context, 'CACHE', root / 'target/cache'), mock.patch.dict('os.environ', {'CARGO_HOME': str(root / 'home')}):
                    first = context.source_snapshot()
                    extra.write_text('[build]\nrustflags = ["--cfg=changed"]')
                    self.assertNotEqual(context.source_snapshot(), first)

    def test_context_comparison_preserves_runtime_inputs_and_rejects_truncation(self):
        import worklist_runtime_context as context
        fields = [b'runtime-dependencies-abi-test'] + [str(index).encode() for index in range(10)]
        encode = lambda values: b''.join(len(value).to_bytes(8, 'little') + value for value in values)
        original = context.support_identity(encode(fields))
        changed = list(fields)
        changed[-1] = b'new caller environment'
        self.assertEqual(context.support_identity(encode(changed)), original)
        changed[6] = b'changed runtime source'
        self.assertNotEqual(context.support_identity(encode(changed)), original)
        for data in [b'broken', encode(fields)[:-1], encode(fields[:-1])]:
            with self.assertRaises(ValueError):
                context.support_identity(data)

class FrontendComparisonTests(unittest.TestCase):
    def test_comparison_preserves_samples_and_rejects_changed_emission(self):
        import contextlib
        import io
        import json
        import worklist_frontend as frontend
        for changed in [False, True]:
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as directory:
                root = pathlib.Path(directory)
                before, after = root / 'before', root / 'after'
                before.write_bytes(b'before compiler')
                after.write_bytes(b'after compiler')
                output_paths = []
                def run(command, project, env):
                    output = pathlib.Path(command[-1])
                    output_paths.append(output)
                    output.write_text('changed Rust' if changed and command[0] == str(after) else 'same Rust')
                    return 42, None, {'load': 1, 'limit': 4}
                captured = io.StringIO()
                with mock.patch.object(frontend, 'PROJECTS', [('worklist', root)]), mock.patch.object(frontend.context, 'source_snapshot', return_value={'source': 'stable'}):
                    with mock.patch.object(frontend.rust_control, 'run', side_effect=run), contextlib.redirect_stdout(captured), contextlib.redirect_stderr(io.StringIO()):
                        if changed:
                            with self.assertRaisesRegex(AssertionError, 'Emitted Rust changed'):
                                frontend.compare(before, after, 3)
                        else:
                            frontend.compare(before, after, 3)
                report = json.loads(captured.getvalue())
                self.assertEqual(len(report['execution']), 3 if changed else 6)
                self.assertEqual(len(report['setup']), 1 if changed else 2)
                self.assertTrue(all(row['durations'] == [42] for row in report['execution']))
                self.assertTrue(all(not path.exists() for path in output_paths))

if __name__ == '__main__':
    unittest.main()
