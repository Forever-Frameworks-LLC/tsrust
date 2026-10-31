import re
import unittest
from pathlib import Path

class PerformanceReadabilityTest(unittest.TestCase):
    def test_cpu_metrics_lead_and_tables_keep_compact_headers(self):
        page = (Path(__file__).parents[1] / 'src/content/docs/project/performance.md').read_text()
        self.assertLess(page.index('## CPU work in an HTTP handler'), page.index('## Language and library summary'))
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

if __name__ == '__main__':
    unittest.main()
