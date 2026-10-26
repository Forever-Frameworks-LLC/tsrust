"""Privacy and exact-measurement checks; no compiler or benchmark processes."""
import json
from pathlib import Path
import sqlite3
import unittest

import analytics
import public
import store
import test_analytics


class PublicTests(unittest.TestCase):
    setUp = test_analytics.AnalyticsTests.setUp
    ingest = test_analytics.AnalyticsTests.ingest

    def test_measurements_survive_without_original_logs_or_private_metadata(self):
        self.raw['machine'] = {'cpu': 'Apple M3', 'platform': 'darwin', 'architecture': 'arm64',
                               'logicalCPUs': 8, 'totalMemoryBytes': 8589934592,
                               'hostname': 'private-laptop', 'serial': 'private-serial',
                               'home': '/Users/private-person', 'ip': '192.168.1.123'}
        self.raw['versions'] = {'node': 'v24.11.1', 'rust': 'rustc 1.96.1 (31fca3adb 2026-06-26)',
                                'custom': 'private-version', 'bun': '/Users/private-version'}
        self.row['notes'] = 'private-note /Users/private-person'
        self.raw['compilerInputs'] = [['/Users/private-person/source.rs', 'd'*64]]
        self.raw['secret'] = 'private-secret'
        self.raw['release'] = {'path': '/Users/private-person/bin', 'sha256': 'e'*64}
        raw = self.root/self.paths[0]
        raw.write_text(json.dumps(self.raw))
        proof = json.loads(self.proof.read_text())
        proof['release'] = self.raw['release']
        proof['records'][0]['sha256'] = analytics.sha(raw)
        self.proof.write_text(json.dumps(proof))
        self.ingest()
        original_sha = analytics.sha(self.db)
        output = self.root/'measurements.sqlite'
        result = public.export(self.db, output)
        self.assertEqual(analytics.sha(self.db), original_sha)
        self.assertEqual(result['counts']['analytics_samples'], 8)
        self.assertEqual(result['counts']['analytics_profiles'], 3)
        self.assertNotIn(b'private-', output.read_bytes())
        self.assertNotIn(b'/Users/', output.read_bytes())
        self.assertNotIn(b'original unmodified evidence', output.read_bytes())
        before = analytics.checkpoint(self.db)
        after = analytics.checkpoint(output)
        self.assertEqual(before['rows'], after['rows'])
        self.assertEqual(after['provenance']['machine']['totalMemoryBytes'], 8589934592)
        self.assertEqual(after['provenance']['compilerInputs'], ['d'*64])
        self.assertEqual(after['provenance']['releaseSha256'], 'e'*64)
        with public.connect(output) as target, store.connect(self.db) as source:
            public.validate(target)
            for table in ['analytics_samples', 'analytics_summaries', 'analytics_profiles']:
                query = f'SELECT * FROM {table} ORDER BY 1,2,3'
                self.assertEqual(source.execute(query).fetchall(), target.execute(query).fetchall())
            with self.assertRaises(sqlite3.OperationalError):
                target.execute('DELETE FROM analytics_samples')

    def test_no_overwrite_or_partial_output_on_invalid_identity(self):
        self.ingest()
        output = self.root/'measurements.sqlite'
        output.write_bytes(b'keep')
        with self.assertRaises(FileExistsError):
            public.export(self.db, output)
        self.assertEqual(output.read_bytes(), b'keep')
        output.unlink()
        with store.connect(self.db, writable=True) as source:
            source.execute('UPDATE analytics_workloads SET context=?',
                           (json.dumps({'emittedSha256': '/Users/private-person'}),))
        with self.assertRaises((ValueError, sqlite3.IntegrityError)):
            public.export(self.db, output)
        self.assertFalse(output.exists())
        self.assertEqual(list(self.root.glob('tmp*.sqlite')), [])

    def test_unknown_metadata_is_omitted_and_malformed_hashes_fail(self):
        self.assertEqual(public.metadata({'machine': {'cpu': 'private-laptop'}, 'logs': 'private-log'}), {})
        with self.assertRaises(ValueError):
            public.metadata({'compilerInputs': [['local.rs', '/Users/private-person']]})
        values = public.metadata({'release': {'sourceCommit': 'a'*40, 'runtimeSourceSha256': 'b'*64,
                                             'path': '/Users/private-person'},
                                  'versions': {'nativeLockSha256': 'c'*64},
                                  'machine': {'cpu': 'Apple M3', 'release': 'private-kernel'}})
        self.assertEqual(public.metadata(values), values)
        self.assertEqual(values['releaseSourceCommit'], 'a'*40)
        self.assertNotIn('private', json.dumps(values))

    def test_bundled_dataset_has_only_public_measurements(self):
        with public.connect(Path(__file__).parents[1]/'measurements.sqlite') as db:
            counts = public.validate(db)
            self.assertGreater(counts['analytics_runs'], 0)
            self.assertGreater(counts['analytics_samples'], 0)


if __name__ == '__main__':
    unittest.main()
