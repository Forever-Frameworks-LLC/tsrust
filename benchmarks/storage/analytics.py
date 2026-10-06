#!/usr/bin/env python3
"""Exact accepted records plus disposable, source-keyed SQLite analytics."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import sys
import tempfile
import store

SCHEMA = '''
CREATE TABLE IF NOT EXISTS analytics_runs (
 raw_sha TEXT PRIMARY KEY REFERENCES blobs(sha256), proof_sha TEXT NOT NULL REFERENCES blobs(sha256),
 kind TEXT NOT NULL, timestamp TEXT NOT NULL, semantic_epoch TEXT NOT NULL,
 measurement_scope TEXT NOT NULL, metadata TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS analytics_workloads (
 raw_sha TEXT NOT NULL REFERENCES analytics_runs(raw_sha), workload TEXT NOT NULL,
 source_sha TEXT NOT NULL, counterpart_sha TEXT, iterations INTEGER NOT NULL,
 operations_per_iteration INTEGER NOT NULL, context TEXT NOT NULL,
 PRIMARY KEY(raw_sha,workload,source_sha));
CREATE TABLE IF NOT EXISTS analytics_samples (
 raw_sha TEXT NOT NULL, workload TEXT NOT NULL, source_sha TEXT NOT NULL,
 engine TEXT NOT NULL, process_index INTEGER NOT NULL, sample_index INTEGER NOT NULL,
 duration_ms REAL NOT NULL CHECK(duration_ms>0), operations INTEGER NOT NULL CHECK(operations>0),
 ops_per_second REAL NOT NULL,
 PRIMARY KEY(raw_sha,workload,source_sha,engine,process_index,sample_index),
 FOREIGN KEY(raw_sha,workload,source_sha) REFERENCES analytics_workloads(raw_sha,workload,source_sha));
CREATE TABLE IF NOT EXISTS analytics_summaries (
 raw_sha TEXT NOT NULL, workload TEXT NOT NULL, source_sha TEXT NOT NULL, engine TEXT NOT NULL,
 samples INTEGER NOT NULL, min_ops_per_second REAL NOT NULL, avg_ops_per_second REAL NOT NULL,
 max_ops_per_second REAL NOT NULL,
 PRIMARY KEY(raw_sha,workload,source_sha,engine),
 FOREIGN KEY(raw_sha,workload,source_sha) REFERENCES analytics_workloads(raw_sha,workload,source_sha));
CREATE TABLE IF NOT EXISTS analytics_profiles (
 raw_sha TEXT NOT NULL, workload TEXT NOT NULL, source_sha TEXT NOT NULL,
 sample_index INTEGER NOT NULL, operations INTEGER NOT NULL,
 allocations INTEGER NOT NULL, reallocations INTEGER NOT NULL, requested_bytes INTEGER NOT NULL,
 retained_bytes INTEGER NOT NULL, peak_bytes INTEGER NOT NULL, input_bytes INTEGER NOT NULL,
 live_after_output INTEGER NOT NULL, live_after_final INTEGER NOT NULL,
 PRIMARY KEY(raw_sha,workload,source_sha,sample_index),
 FOREIGN KEY(raw_sha,workload,source_sha) REFERENCES analytics_workloads(raw_sha,workload,source_sha));
CREATE INDEX IF NOT EXISTS analytics_source_lookup ON analytics_summaries(source_sha,workload,engine);
CREATE VIEW IF NOT EXISTS analytics_profile_resources AS
 SELECT raw_sha,workload,source_sha,sample_index,operations,
 allocations*1.0/operations AS allocations_per_operation,
 reallocations*1.0/operations AS reallocations_per_operation,
 requested_bytes*1.0/operations AS requested_bytes_per_operation,
 retained_bytes*1.0/operations AS retained_bytes_per_operation,
 peak_bytes*1.0/operations AS retained_batch_peak_bytes_per_operation,
 input_bytes AS input_adaptation_bytes,live_after_output,live_after_final
 FROM analytics_profiles;
'''


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(store.CHUNK), b''):
            h.update(block)
    return h.hexdigest()


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False)


def relative(root, path):
    path = Path(path)
    path = path if path.is_absolute() else root / path
    name = path.resolve(strict=True).relative_to(root).as_posix()
    store.safe_parts(name)
    return name


def positive_int(value, name):
    if type(value) is not int or value <= 0:
        raise ValueError('positive integer required: ' + name)
    return value


def derive(database, relative_path, digest, proof_digest, proof):
    # Decode/verify the original archive into a temporary file; load one record only.
    with tempfile.TemporaryFile() as original:
        read_db = store.connect(database)
        try:
            row = read_db.execute('SELECT id,blobs.sha256,original_size FROM records JOIN blobs USING(sha256) WHERE path=?', (relative_path,)).fetchone()
            if row is None or row[1] != digest:
                raise ValueError('original archive binding mismatch')
            store.decode_blob(read_db, row, original)
        finally:
            read_db.close()
        original.seek(0)
        data = json.load(original)
    profile = data.get('kind') == 'native-workload-allocation-diagnosis'
    if not profile and (data.get('schemaVersion') != 1 or data.get('verifiedOnly') is not False or not data.get('execution')):
        raise ValueError('only accepted measured records produce throughput analytics')
    scope = data.get('measurementScope', 'native-allocation-profile' if profile else 'runtime-startup-compilation')
    if scope == 'correctness-only':
        raise ValueError('correctness-only records are not measurements')
    epoch = data.get('semanticEpoch', data['manifest'].get('semanticEpoch', 'historical-utf16-unversioned'))
    metadata = {key: data[key] for key in ['measuredEngines', 'measurementPlanSha256', 'machine', 'versions', 'nodeComponents', 'rustVersion', 'compilerInputs', 'oracleInputs', 'harnessSha256', 'adapterSha256', 'lockSha256', 'numberAdapterInputs', 'sliceAdapterInputs'] if key in data}
    metadata['release'] = proof.get('release')
    workloads = {row['id']: row for row in data['manifest']['workloads']}
    sources = {row['workload' if profile else 'id']: row for row in data['rows' if profile else 'sources']}
    db = store.connect(database, writable=True)
    try:
        db.executescript(SCHEMA)
        db.execute('BEGIN IMMEDIATE')
        existing = db.execute('SELECT proof_sha FROM analytics_runs WHERE raw_sha=?', (digest,)).fetchone()
        if existing:
            if existing[0] != proof_digest:
                raise ValueError('accepted proof binding conflict for indexed original record')
            db.execute('ROLLBACK')
            return {'alreadyIndexed': True}
        db.execute('INSERT INTO analytics_runs VALUES(?,?,?,?,?,?,?)', (digest, proof_digest, 'profile' if profile else 'execution', data['timestamp'], epoch, scope, encode(metadata)))
        for id, source in sources.items():
            row = workloads[id]
            source_sha = source['sourceSha256' if profile else 'tsrSha256']
            counterpart_sha = source.get('tsSha256', row.get('counterpartSha256'))
            if row.get('sourceSha256', source_sha) != source_sha or row.get('counterpartSha256', counterpart_sha) != counterpart_sha:
                raise ValueError('captured manifest/source identity mismatch: ' + id)
            iterations = positive_int(row['iterations'], 'iterations')
            ops = positive_int(row.get('operationsPerIteration', 1), 'operationsPerIteration')
            context = {key: row[key] for key in ['result', 'resultEncoding', 'operationUnit', 'notes', 'specification', 'parameters'] if key in row}
            context['emittedSha256'] = source.get('emittedSha256')
            db.execute('INSERT INTO analytics_workloads VALUES(?,?,?,?,?,?,?)', (digest, id, source_sha, counterpart_sha, iterations, ops, encode(context)))
        samples = 0
        if profile:
            for row in data['rows']:
                id = row['workload'];source_sha = row['sourceSha256'];workload = workloads[id]
                if row['iterations'] != workload['iterations']:
                    raise ValueError('profile iteration mismatch')
                operations = row['iterations'] * workload.get('operationsPerIteration', 1)
                for index, sample in enumerate(row['samples']):
                    keys = ['allocations', 'reallocations', 'requestedBytes', 'retainedRequestedBytes', 'peakRequestedBytes', 'inputRequestedBytes', 'liveBytesAfterOutputDrop', 'liveBytesAfterDrop']
                    values = [sample[key] for key in keys]
                    if any(type(value) is not int or value < 0 for value in values) or values[-2:] != [0, 0]:
                        raise ValueError('invalid resource counters or nonzero disposal')
                    db.execute('INSERT INTO analytics_profiles VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)', (digest, id, source_sha, index, operations, *values));samples += 1
        else:
            for cell in data['execution']:
                id = cell['workload'];row = workloads[id];source_sha = sources[id]['tsrSha256']
                process = cell['processIndex']
                if type(process) is not int or process < 0 or not cell['durations']:
                    raise ValueError('invalid measured process/sample boundary')
                operations = row['iterations'] * row.get('operationsPerIteration', 1)
                for index, duration in enumerate(cell['durations']):
                    if type(duration) not in (int, float) or not math.isfinite(duration) or duration <= 0:
                        raise ValueError('invalid measured duration')
                    rate = operations * 1000 / duration
                    if not math.isfinite(rate):
                        raise ValueError('invalid throughput')
                    db.execute('INSERT INTO analytics_samples VALUES(?,?,?,?,?,?,?,?,?)', (digest, id, source_sha, cell['engine'], process, index, duration, operations, rate));samples += 1
            db.execute('INSERT INTO analytics_summaries SELECT raw_sha,workload,source_sha,engine,count(*),min(ops_per_second),avg(ops_per_second),max(ops_per_second) FROM analytics_samples WHERE raw_sha=? GROUP BY raw_sha,workload,source_sha,engine', (digest,))
        db.execute('COMMIT')
        return {'workloads': len(sources), 'samples': samples, 'kind': 'profile' if profile else 'execution'}
    except BaseException:
        if db.in_transaction:
            db.execute('ROLLBACK')
        raise
    finally:
        db.close()


def ingest(database, root, proof_path, expected_proof_sha):
    root = Path(root).resolve(strict=True)
    proof_name = relative(root, proof_path)
    if sha(root / proof_name) != expected_proof_sha:
        raise ValueError('accepted proof SHA mismatch')
    proof = json.loads((root / proof_name).read_bytes())
    if proof.get('runtimeAccepted') is not True or proof.get('profilesAccepted') is not True or proof.get('timed') is not True:
        raise ValueError('actual accepted runtime/profile proof required')
    records = []
    for item in proof['records']:
        name = relative(root, item['path'])
        if not (name.startswith('runtime/results/') or name.startswith('runtime/diagnostics/')):
            raise ValueError('expected runtime result/profile path')
        if sha(root / name) != item['sha256']:
            raise ValueError('original accepted raw record drift: ' + name)
        records.append((name, item['sha256']))
    if not records or len(set(name for name, _ in records)) != len(records):
        raise ValueError('unique accepted raw record bindings required')
    store.import_records(database, root, [proof_name] + [name for name, _ in records])
    indexed = []
    for name, digest in records:
        indexed.append({'path': name, 'sha256': digest, **derive(database, name, digest, expected_proof_sha, proof)})
    return {'proof': {'path': proof_name, 'sha256': expected_proof_sha}, 'indexed': indexed, 'looseOriginalsRemoved': False}


def checkpoint(database, raw_sha=None):
    db = store.connect(database)
    try:
        if raw_sha is None:
            row = db.execute("SELECT raw_sha FROM analytics_runs WHERE kind='execution' ORDER BY timestamp DESC,raw_sha DESC LIMIT 1").fetchone()
            if row is None:
                raise ValueError('no accepted measured run indexed')
            raw_sha = row[0]
        run = db.execute("SELECT timestamp,semantic_epoch,measurement_scope,metadata FROM analytics_runs WHERE raw_sha=? AND kind='execution'", (raw_sha,)).fetchone()
        if run is None:
            raise ValueError('unknown accepted measured run')
        rows = []
        for id, source, counterpart, n, ops, low, avg, high, count, nlow, navg, nhigh, ncount in db.execute('''
 SELECT w.workload,w.source_sha,w.counterpart_sha,w.iterations,w.operations_per_iteration,
 n.min_ops_per_second,n.avg_ops_per_second,n.max_ops_per_second,n.samples,
 j.min_ops_per_second,j.avg_ops_per_second,j.max_ops_per_second,j.samples
 FROM analytics_workloads w JOIN analytics_summaries n USING(raw_sha,workload,source_sha)
 JOIN analytics_summaries j USING(raw_sha,workload,source_sha)
 WHERE w.raw_sha=? AND n.engine='native' AND j.engine='node-tsc' ORDER BY w.workload''', (raw_sha,)):
            rows.append({'id': id, 'sourceSha256': source, 'counterpartSha256': counterpart, 'operationsPerBatch': n * ops,
                         'native': {'samples': count, 'min': low, 'avg': avg, 'max': high},
                         'compiledNode': {'samples': ncount, 'min': nlow, 'avg': navg, 'max': nhigh}, 'nativeNodeRatio': avg / navg, 'signal': 'review-native-slower' if avg < navg else 'native-at-least-as-fast'})
        return {'rawSha256': raw_sha, 'timestamp': run[0], 'semanticEpoch': run[1], 'measurementScope': run[2],
                'provenance': json.loads(run[3]), 'comparedRows': len(rows), 'nativeWins': sum(row['nativeNodeRatio'] > 1 for row in rows), 'rows': rows,
                'policy': 'Same accepted original run and captured source pair; no cross-run/toolchain/hardware aggregation. Rates are reciprocal per actual sample; complete workload scope applies.'}
    finally:
        db.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__);sub = parser.add_subparsers(dest='command', required=True)
    add = sub.add_parser('ingest');add.add_argument('database');add.add_argument('--root', required=True);add.add_argument('--proof', required=True);add.add_argument('--proof-sha256', required=True)
    query = sub.add_parser('checkpoint');query.add_argument('database');query.add_argument('--raw-sha256')
    args = parser.parse_args()
    result = ingest(args.database, args.root, args.proof, args.proof_sha256) if args.command == 'ingest' else checkpoint(args.database, args.raw_sha256)
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
