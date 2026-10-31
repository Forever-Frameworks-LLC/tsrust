import json
from pathlib import Path
import sqlite3
import tempfile
import unittest

import measurements


def results(at, rows):
    return {'timestamp': at, 'manifest': {'workloads': [
        {'id': 'w', 'iterations': 100, 'operationsPerIteration': 2}]},
        'execution': [{'workload': 'w', 'engine': engine, 'durations': durations} for engine, durations in rows]}


class MeasurementsTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.db = measurements.connect(self.root / 'm.sqlite')

    def tearDown(self):
        self.db.close()
        self.directory.cleanup()

    def write(self, name, data):
        path = self.root / name
        path.write_text(json.dumps(data))
        return path

    def history(self, engine):
        row = self.db.execute('SELECT history FROM measurements WHERE workload=? AND engine=?', ('w', engine)).fetchone()
        return json.loads(row[0])

    def test_ingest_keeps_one_row_per_engine_and_skips_repeated_runs(self):
        first = self.write('a.json', results('2026-01-01T00:00:00Z', [('native', [1.0, 2.0, 4.0]), ('node-tsc', [4.0])]))
        second = self.write('b.json', results('2026-01-02T00:00:00Z', [('native', [2.0])]))
        self.assertEqual(measurements.ingest(self.db, [second, first]), 3)
        self.assertEqual(measurements.ingest(self.db, [first]), 0)
        self.assertEqual(self.db.execute('SELECT count(*) FROM measurements').fetchone()[0], 2)
        native = self.history('native')
        self.assertEqual([entry['at'] for entry in native], ['2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z'])
        # 200 operations per sample: 200,000, 100,000 and 50,000 ops/s.
        self.assertEqual(native[0], {'at': '2026-01-01T00:00:00Z', 'ms': 2.0, 'ops': 116666.667, 'samples': 3})

    def test_latest_pairs_engines_from_the_same_run(self):
        self.write('a.json', results('2026-01-01T00:00:00Z', [('native', [1.0]), ('node-tsc', [4.0])]))
        self.write('b.json', results('2026-01-02T00:00:00Z', [('native', [2.0])]))
        measurements.ingest(self.db, sorted(self.root.glob('*.json')))
        [row] = measurements.latest(self.db, 'node-tsc')
        self.assertEqual(row['at'], '2026-01-01T00:00:00Z')
        self.assertEqual(row['ratio'], 4.0)

    def test_verification_only_results_are_not_measurements(self):
        data = results('2026-01-01T00:00:00Z', [('native', [1.0])])
        data['verifiedOnly'] = True
        self.assertEqual(measurements.ingest(self.db, [self.write('v.json', data)]), 0)

    def test_migrate_summarizes_previous_samples(self):
        old = sqlite3.connect(self.root / 'old.sqlite')
        old.executescript('''CREATE TABLE analytics_runs (raw_sha TEXT, kind TEXT, timestamp TEXT);
            CREATE TABLE analytics_samples (raw_sha TEXT, workload TEXT, engine TEXT, duration_ms REAL, ops_per_second REAL);
            INSERT INTO analytics_runs VALUES ('r', 'execution', '2026-01-01T00:00:00Z'), ('p', 'profile', '2026-01-02T00:00:00Z');
            INSERT INTO analytics_samples VALUES ('r', 'w', 'native', 1.0, 10.0), ('r', 'w', 'native', 3.0, 30.0), ('p', 'w', 'native', 9.0, 1.0);''')
        old.commit()
        old.close()
        self.assertEqual(measurements.migrate(self.db, self.root / 'old.sqlite'), 1)
        self.assertEqual(self.history('native'), [{'at': '2026-01-01T00:00:00Z', 'ms': 2.0, 'ops': 20.0, 'samples': 2}])


if __name__ == '__main__':
    unittest.main()
