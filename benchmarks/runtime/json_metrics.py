"""Derive JSON throughput/latency from immutable SQLite records; never run workloads."""
import hashlib
import io
import json
import math
from pathlib import Path
import statistics
import sys


def summarize(samples, byte_count):
    if len(samples) != 3 or any(len(s['durations']) != 31 for s in samples):
        raise ValueError('Expected three processes and 31 batches')
    counts = {s['iterations'] for s in samples}
    if len(counts) != 1 or next(iter(counts)) <= 0 or byte_count < 0:
        raise ValueError('Invalid operation or byte count')
    values = sorted(v for s in samples for v in s['durations'])
    if any(not math.isfinite(v) or v <= 0 for v in values):
        raise ValueError('Invalid elapsed time')
    iterations = next(iter(counts))
    median = statistics.median(values)
    return {'processes': 3, 'batches': len(values), 'operationsPerBatch': iterations,
            'offeredBytesPerOperation': byte_count, 'p50BatchMs': median,
            'p95BatchMs': values[math.ceil(.95 * len(values)) - 1],
            'p50OperationMs': median / iterations,
            'medianOfferedBytesPerSecond': byte_count * iterations * 1000 / median}


def main():
    root = Path(__file__).resolve().parents[2]
    sys.path.insert(0, str(root / 'benchmarks/storage'))
    import store
    records = [('encoding', 'tsrust-json-encoding-opt-timing-b162df03.json'),
               ('decoding', 'tsrust-json-decoding-timing-ec9c4b41.json'),
               ('streaming', 'tsrust-json-stream-timing-ad4f3224.json'),
               ('raw', 'tsrust-json-raw-timing-07331fe6.json'),
               ('raw-selective', 'tsrust-json-selective-timing-510cae62.json')]
    rows, inputs = [], []
    for family, name in records:
        data = io.BytesIO()
        store.read_record(root / 'benchmarks/records.sqlite', name, data)
        raw = data.getvalue()
        record = json.loads(raw)
        inputs.append({'record': name, 'sha256': hashlib.sha256(raw).hexdigest()})
        for case in record['rows']:
            count = case['inputBytes']
            if family == 'streaming':
                count = len(b'{"active":true,"name":"hello","values":[20,17]}\n') * case['n'] if 'write_' in case['id'] else case['payloadBytes']
            for engine in ['native', 'directRust', 'node']:
                samples = [s for s in case['samples'] if s['engine'] == engine]
                if {s['processIndex'] for s in samples} != {0, 1, 2}:
                    raise ValueError('Missing process identity')
                if any(s['checksum'] != case['expected'] for s in samples):
                    raise ValueError('Incorrect observed result')
                rows.append({'family': family, 'case': case['id'], 'size': case['n'],
                             'engine': engine, **summarize(samples, count)})
    print(json.dumps({'schemaVersion': 1, 'scope': 'derived-json-timing',
          'method': 'Nearest-rank p95 over 93 retained batches; byte rate uses median batch latency. Bytes are offered input for decoding and raw validation (including failed input and ignored fields), exact encoded output for encoding, and complete file bytes for streaming. No claim that rejected input was fully consumed. No samples combined across source records. Cold samples stay in original records.',
          'inputs': inputs, 'scriptSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'rows': rows}, indent=2))


if __name__ == '__main__':
    main()
