import hashlib
import io
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import unittest
import zlib

import store


class StoreTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='tsrust-store-test-')
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.source = self.base / 'source'
        self.destination = self.base / 'destination'
        self.source.mkdir()
        self.destination.mkdir()
        self.database = self.base / 'archive.sqlite'

    def write(self, relative, data):
        target = self.source / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)

    def load(self, *paths):
        return store.import_records(self.database, self.source, paths)

    def test_exact_roundtrip_json_binary_unicode_and_large_streams(self):
        fixtures = {
            'results/測定.json': '{\n  "label": "é😀", "number": -0.0\n}\n'.encode(),
            'diagnostics/native': bytes(range(256)) * 1700,
            'diagnostics/empty': b'',
            'approximate-references/repeated.json': b'  [0.123456789]\n' * 100000,
        }
        for path, data in fixtures.items():
            self.write(path, data)
        self.assertEqual(self.load(*fixtures), 4)
        self.assertEqual(store.verify(self.database), 4)
        rows = {row['path']: row for row in store.list_records(self.database)}
        for path, data in fixtures.items():
            store.export_record(self.database, path, self.destination)
            self.assertEqual((self.destination / path).read_bytes(), data)
            self.assertEqual(rows[path]['sha256'], hashlib.sha256(data).hexdigest())
            self.assertEqual(rows[path]['original_size'], len(data))
        self.assertEqual(rows['results/測定.json']['kind'], 'result')

    def test_duplicate_bytes_share_blob_and_reimport_is_idempotent(self):
        for path in ('diagnostics/a', 'results/b'):
            self.write(path, b'same original bytes\n')
        self.load('diagnostics', 'results')
        self.load('diagnostics/a')
        self.assertEqual(store.verify(self.database), 1)
        self.assertEqual(len(list(store.list_records(self.database))), 2)

    def test_changed_path_rolls_back_entire_import(self):
        self.write('original.json', b'original')
        self.load('original.json')
        self.write('new.json', b'new')
        self.write('original.json', b'changed')
        with self.assertRaisesRegex(ValueError, 'immutable path conflict'):
            self.load('new.json', 'original.json')
        self.assertEqual([r['path'] for r in store.list_records(self.database)], ['original.json'])
        self.assertEqual(store.verify(self.database), 1)

    def test_kind_binding_cannot_change(self):
        self.write('original', b'original')
        self.load('original')
        with self.assertRaisesRegex(ValueError, 'immutable path conflict'):
            store.import_records(self.database, self.source, ['original'], kind='result')

    def poison(self, replacement):
        # Simulate disk/database damage, bypassing append-only SQL update triggers.
        db = sqlite3.connect(self.database)
        self.addCleanup(db.close)
        db.execute('DROP TRIGGER immutable_blobs_update')
        db.execute('UPDATE blobs SET compressed=?', (replacement,))
        db.commit()

    def test_corruption_rejected_by_verify_export_and_duplicate_import(self):
        self.write('result.json', b'original content')
        self.load('result.json')
        self.poison(zlib.compress(b'tampered content'))
        for operation in (lambda: store.verify(self.database),
                          lambda: store.export_record(self.database, 'result.json', self.destination),
                          lambda: self.load('result.json')):
            with self.assertRaises((ValueError, zlib.error)):
                operation()
        self.assertEqual(list(self.destination.iterdir()), [])

    def test_truncated_compression_rejected(self):
        self.write('result', b'original')
        self.load('result')
        self.poison(zlib.compress(b'original')[:-1])
        with self.assertRaisesRegex(ValueError, 'corrupt blob'):
            store.verify(self.database)

    def test_trailing_compression_rejected(self):
        self.write('result', b'original')
        self.load('result')
        self.poison(zlib.compress(b'original') + b'junk')
        with self.assertRaisesRegex(ValueError, 'trailing compressed data'):
            store.verify(self.database)

    def test_no_overwrite_of_file_or_dangling_symlink(self):
        self.write('result', b'original')
        self.load('result')
        target = self.destination / 'result'
        target.write_bytes(b'existing')
        with self.assertRaises(FileExistsError):
            store.export_record(self.database, 'result', self.destination)
        self.assertEqual(target.read_bytes(), b'existing')
        target.unlink()
        target.symlink_to(self.base / 'absent')
        with self.assertRaises(FileExistsError):
            store.export_record(self.database, 'result', self.destination)
        self.assertFalse((self.base / 'absent').exists())
        self.assertEqual([p.name for p in self.destination.iterdir()], ['result'])

    def test_traversal_and_absolute_paths_rejected(self):
        self.write('valid', b'original')
        self.load('valid')
        for path in ('../outside', '/absolute', 'a/../valid', 'a//valid', './valid', 'a\\valid'):
            with self.subTest(path=path):
                with self.assertRaises(ValueError):
                    self.load(path)
                with self.assertRaises(ValueError):
                    store.export_record(self.database, path, self.destination)

    def test_import_and_export_symlink_escape_rejected(self):
        outside = self.base / 'outside'
        outside.mkdir()
        (outside / 'record').write_bytes(b'outside')
        (self.source / 'link').symlink_to(outside, target_is_directory=True)
        (self.source / 'file-link').symlink_to(outside / 'record')
        for path in ('link', 'link/record', 'file-link'):
            with self.assertRaises((ValueError, OSError)):
                self.load(path)
        self.write('nested/record', b'original')
        self.load('nested')
        (self.destination / 'nested').symlink_to(outside, target_is_directory=True)
        with self.assertRaises(OSError):
            store.export_record(self.database, 'nested/record', self.destination)
        self.assertEqual((outside / 'record').read_bytes(), b'outside')

    def test_oversized_decoded_blob_rejected(self):
        self.write('result', b'small')
        self.load('result')
        self.poison(zlib.compress(b'x' * (store.CHUNK * 3)))
        with self.assertRaisesRegex(ValueError, 'oversized output'):
            store.export_record(self.database, 'result', self.destination)
        self.assertEqual(list(self.destination.iterdir()), [])

    def test_cli_commands_and_error_exit(self):
        self.write('results/example.json', b'{"original": true}\n')
        executable = Path(store.__file__)

        def command(*args):
            return subprocess.run([sys.executable, '-B', str(executable), *map(str, args)],
                                  text=True, capture_output=True, check=False)

        imported = command('import', self.database, 'results', '--root', self.source)
        self.assertEqual(imported.returncode, 0, imported.stderr)
        verified = command('verify', self.database)
        self.assertEqual(verified.returncode, 0, verified.stderr)
        listed = command('list', self.database)
        self.assertEqual(listed.returncode, 0, listed.stderr)
        self.assertIn('results/example.json', listed.stdout)
        exported = command('export', self.database, 'results/example.json', '--root', self.destination)
        self.assertEqual(exported.returncode, 0, exported.stderr)
        rejected = command('export', self.database, '../escape', '--root', self.destination)
        self.assertEqual(rejected.returncode, 1)
        self.assertIn('without traversal', rejected.stderr)

    def test_verified_read_emits_exact_original_and_no_corrupt_output(self):
        raw = b'{ "value": -0.0, "label": "\\u00e9" }\n\n'
        self.write('results/original.json', raw)
        self.load('results')
        output = io.BytesIO()
        store.read_record(self.database, 'results/original.json', output)
        self.assertEqual(output.getvalue(), raw)
        self.poison(zlib.compress(b'corrupt'))
        rejected = io.BytesIO()
        with self.assertRaises(ValueError):
            store.read_record(self.database, 'results/original.json', rejected)
        self.assertEqual(rejected.getvalue(), b'')

    def test_restore_original_relative_tree_and_prefix(self):
        fixtures = {'runtime/results/run.json': b'{"genuine":true}\n',
                    'runtime/diagnostics/nested/native': bytes(range(256)),
                    'runtime/diagnostics/\u6e2c\u5b9a.json': b'original unicode filename'}
        for path, raw in fixtures.items():
            self.write(path, raw)
        self.load('runtime')
        self.assertEqual(store.restore_tree(self.database, self.destination, 'runtime/diagnostics'), 2)
        self.assertFalse((self.destination / 'runtime/results').exists())
        self.assertEqual(store.restore_tree(self.database, self.destination, 'runtime/results'), 1)
        for path, raw in fixtures.items():
            self.assertEqual((self.destination / path).read_bytes(), raw)
        with self.assertRaises(FileExistsError):
            store.restore_tree(self.database, self.destination)

    def test_read_and_restore_cli_use_exact_bytes_and_original_tree(self):
        raw = b'{ "exact": true }\n\n'
        self.write('runtime/results/run.json', raw)
        self.load('runtime')
        executable = Path(store.__file__)
        result = subprocess.run([sys.executable, '-B', str(executable), 'read',
                                 str(self.database), 'runtime/results/run.json'], capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, raw)
        restored = subprocess.run([sys.executable, '-B', str(executable), 'restore',
                                   str(self.database), '--root', str(self.destination)], capture_output=True)
        self.assertEqual(restored.returncode, 0, restored.stderr)
        self.assertEqual((self.destination / 'runtime/results/run.json').read_bytes(), raw)

    def test_sql_bindings_and_blobs_are_append_only(self):
        self.write('result', b'original')
        self.load('result')
        with sqlite3.connect(self.database) as db:
            for command in ('DELETE FROM blobs', 'DELETE FROM records',
                            "UPDATE records SET kind='changed'", 'UPDATE blobs SET original_size=1'):
                with self.assertRaises(sqlite3.IntegrityError):
                    db.execute(command)

    def test_unrelated_database_and_archive_self_import_rejected(self):
        other = self.base / 'other.sqlite'
        with sqlite3.connect(other) as db:
            db.execute('CREATE TABLE unrelated(value)')
        with self.assertRaisesRegex(ValueError, 'unrelated database'):
            store.import_records(other, self.source, ['missing'])
        nested = self.source / 'archive.sqlite'
        with self.assertRaisesRegex(ValueError, 'cannot import'):
            store.import_records(nested, self.source, ['archive.sqlite'])


if __name__ == '__main__':
    unittest.main()
