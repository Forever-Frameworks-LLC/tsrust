"""Focused scratch-only checks; run after benchmark CPU exclusivity ends."""
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest import mock

HERE = Path(__file__).parent
STORE = HERE / 'store.py'
spec = importlib.util.spec_from_file_location('compact_closed_gate', HERE / 'compact_closed_gate.py')
compact = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compact)
store_spec = importlib.util.spec_from_file_location('store_fixture', STORE)
store = importlib.util.module_from_spec(store_spec)
store_spec.loader.exec_module(store)


class ClosedGateCompaction(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='tsrust-compaction-test-')
        self.addCleanup(self.temporary.cleanup)
        self.case = Path(self.temporary.name)
        self.root = self.case / 'benchmarks'
        self.root.mkdir()
        self.database = self.root / 'records.sqlite'
        self.output = self.case / 'evidence'
        self.selection = 'runtime/diagnostics/fixture-closed-source-validation-artifacts'
        self.tree = self.root / self.selection
        self.tree.mkdir(parents=True)
        self.originals = {
            self.selection + '/a-first.json': '{"text":"東京 😀"}\n'.encode(),
            self.selection + '/sub/duplicate.json': '{"text":"東京 😀"}\n'.encode(),
            self.selection + '/z-second.json': b'\x00\xff\x80\x00binary\n',
        }
        for relative, content in self.originals.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        self.proof = self.root / 'runtime/diagnostics/completed-proof.json'
        self.proof.write_bytes(b'{"fixtureClosedGate":true,"profiles":48}\n')
        self.proof_bytes = self.proof.read_bytes()
        self.plan = self.case / 'plan.json'
        self.write_plan([self.selection])

    def write_plan(self, paths):
        self.plan.write_text(json.dumps({
            'closedProof': {'path': str(self.proof), 'sha256': hashlib.sha256(self.proof_bytes).hexdigest()},
            'paths': paths,
        }))

    def run_compaction(self, output=None):
        # Keep any intentionally retained rollback scratch within this test case.
        original_mkdtemp = tempfile.mkdtemp
        with mock.patch.object(compact.tempfile, 'mkdtemp',
                               side_effect=lambda **kwargs: original_mkdtemp(dir=self.case, **kwargs)):
            with contextlib.redirect_stdout(io.StringIO()):
                compact.compact(STORE, self.root, self.database, self.plan, output or self.output)

    def assert_originals(self):
        for relative, content in self.originals.items():
            self.assertEqual((self.root / relative).read_bytes(), content)
        self.assertEqual(self.proof.read_bytes(), self.proof_bytes)

    def test_success_dedup_and_exact_tree_restoration(self):
        self.run_compaction()
        self.assertFalse(self.tree.exists())
        self.assertEqual(self.proof.read_bytes(), self.proof_bytes)
        with sqlite3.connect(self.database) as db:
            self.assertEqual(db.execute('SELECT count(*) FROM records').fetchone()[0], 3)
            self.assertEqual(db.execute('SELECT count(*) FROM blobs').fetchone()[0], 2)
        restored = self.case / 'restored'
        restored.mkdir()
        self.assertEqual(store.restore_tree(self.database, restored, self.selection), 3)
        paths = sorted(path.relative_to(restored).as_posix() for path in restored.rglob('*') if path.is_file())
        self.assertEqual(paths, sorted(self.originals))
        for relative, content in self.originals.items():
            self.assertEqual((restored / relative).read_bytes(), content)
        manifest = json.loads((self.output / 'migration.json').read_text())
        self.assertEqual(manifest['files'], 3)
        self.assertEqual(manifest['originalLogicalBytes'], sum(map(len, self.originals.values())))
        self.assertEqual(manifest['database']['sha256'], hashlib.sha256(self.database.read_bytes()).hexdigest())
        self.assertTrue((self.output / 'verification.json').exists())
        self.assertFalse(list(self.case.glob('tsrust-closed-gate-restore-*')))

    def test_injected_removal_failure_restores_every_original(self):
        original_unlink = compact.os.unlink
        failed = False
        def injected(path, *args, **kwargs):
            nonlocal failed
            if path == 'z-second.json' and kwargs.get('dir_fd') is not None and not failed:
                failed = True
                raise OSError('injected selected-leaf removal failure')
            return original_unlink(path, *args, **kwargs)
        with mock.patch.object(compact.os, 'unlink', side_effect=injected):
            with self.assertRaisesRegex(OSError, 'injected selected-leaf'):
                self.run_compaction()
        self.assertTrue(failed)
        self.assert_originals()
        self.assertFalse((self.output / 'migration.json').exists())
        self.assertTrue((self.output / 'verification.json').exists())
        self.assertEqual(store.verify(self.database), 2)
        self.assertTrue(list(self.case.glob('tsrust-closed-gate-restore-*')))

    def test_reject_nonartifact_and_traversal_selections_before_import(self):
        for selection in ['runtime/results', 'runtime/diagnostics/completed-proof.json',
                          '../outside-artifacts', '/tmp/outside-artifacts']:
            with self.subTest(selection=selection):
                self.write_plan([selection])
                with self.assertRaises(ValueError):
                    self.run_compaction()
                self.assertFalse(self.database.exists())
                self.assertFalse(self.output.exists())
                self.assert_originals()

    def test_reject_evidence_output_inside_selected_tree(self):
        with self.assertRaisesRegex(ValueError, 'outside selected artifact trees'):
            self.run_compaction(self.tree / 'evidence')
        self.assertFalse(self.database.exists())
        self.assertFalse((self.tree / 'evidence').exists())
        self.assert_originals()

    def test_reject_existing_evidence_output(self):
        self.output.mkdir()
        sentinel = self.output / 'unchanged'
        sentinel.write_bytes(b'keep this evidence')
        with self.assertRaises(FileExistsError):
            self.run_compaction()
        self.assertEqual(sentinel.read_bytes(), b'keep this evidence')
        self.assertFalse(self.database.exists())
        self.assert_originals()

    def test_reject_closed_proof_drift(self):
        self.proof.write_bytes(b'changed closed proof')
        with self.assertRaisesRegex(ValueError, 'closed-gate proof drift'):
            self.run_compaction()
        self.assertFalse(self.database.exists())
        self.assertFalse(self.output.exists())
        for relative, content in self.originals.items():
            self.assertEqual((self.root / relative).read_bytes(), content)


if __name__ == '__main__':
    unittest.main()
