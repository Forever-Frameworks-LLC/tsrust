import re
import unittest
from pathlib import Path

class PerformanceReadabilityTest(unittest.TestCase):
    def test_microbenchmarks_precede_http_and_tables_keep_compact_headers(self):
        page = (Path(__file__).parents[1] / 'src/content/docs/project/performance.md').read_text()
        self.assertLess(page.index('## Built-in and language microbenchmarks'), page.index('## HTTP benchmarks'))
        self.assertLess(page.index('#### Strings'), page.index('## HTTP benchmarks'))
        self.assertLess(page.index('### Results by group'), page.index('## HTTP benchmarks'))
        self.assertIn('performance-metrics', page)
        self.assertIn('data-label="p99 (ms)"', page)
        self.assertIn('data-label="CPU/req (ms)"', page)
        self.assertIn('role="rowheader">Tsrust', page)
        self.assertNotIn('Median batch ms', page)
        lines = page.splitlines()
        headers = [line for i, line in enumerate(lines[:-1]) if line.startswith('|') and re.match(r'\|\s*---', lines[i + 1])]
        for header in headers:
            columns = header.strip('|').split('|')
            self.assertLessEqual(len(columns), 5, header)
            self.assertNotIn('date', header.lower(), header)
        self.assertEqual(sum(line == '| Workload | Tsrust | Node.js | Rust |' for line in lines), 2)
        self.assertIn('11,904 individual requests', page)
        self.assertIn('Resource polling is outside latency/throughput timing', page)

class CpuExampleTest(unittest.TestCase):
    def test_highlight_shows_the_actual_measured_function_and_selected_ratio(self):
        import json
        import sqlite3
        import hashlib
        root = Path(__file__).resolve().parents[2]
        config = json.loads((root / 'website/tools/performance-runs.json').read_text())
        pin = config['cpuHighlight']
        manifest = json.loads((root / 'benchmarks/runtime/manifest.json').read_text())
        workload = next(x for x in manifest['workloads'] if x['id'] == pin['workload'])
        self.assertEqual((workload['iterations'], manifest['seed']), (pin['iterations'], pin['seed']))
        source = (root / ('benchmarks/runtime/workloads/' + pin['workload'] + '.tsr')).read_text()
        counterpart = (root / ('benchmarks/runtime/workloads/' + pin['workload'] + '.ts')).read_text()
        self.assertEqual(hashlib.sha256(source.encode()).hexdigest(), pin['sourceSha256'])
        self.assertEqual(hashlib.sha256(counterpart.encode()).hexdigest(), pin['counterpartSha256'])
        page = (root / 'website/src/content/docs/project/performance.md').read_text()
        self.assertIn(source.split('\nfunction main()')[0].strip(), page)
        db = sqlite3.connect(root / 'benchmarks/measurements.sqlite')
        rates = {}
        for engine in ['native', 'node-tsc']:
            history = {x['at']: x for x in json.loads(db.execute('SELECT history FROM measurements WHERE workload=? AND engine=?', (pin['workload'], engine)).fetchone()[0])}
            rates[engine] = [history[at] for at in sorted(config['runs']) if at in history][-1]['ops']
        db.close()
        self.assertIn('**{:.2f}× Node.js throughput**'.format(rates['native'] / rates['node-tsc']), page)
        self.assertIn('selected callback microbenchmark', page)

if __name__ == '__main__':
    unittest.main()
