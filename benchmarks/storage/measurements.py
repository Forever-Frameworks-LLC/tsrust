#!/usr/bin/env python3
"""Benchmark measurements: one row per workload and engine, with a short history.

Each row keeps a JSON array of runs, oldest first. An entry records when the run
happened and how long the workload took:

    {"at": "2026-10-30T22:11:30.207Z", "ms": 0.394, "ops": 5.1e6, "samples": 93}

`at` is the run timestamp (entries from the same results file share it, which is
how native and Node.js rows are paired), `ms` is the median sample duration,
`ops` the mean operations per second over the samples and `samples` how many
timed samples the run took.

    measurements.py ingest [results.json ...]   add runs (default: every file in runtime/results)
    measurements.py latest [--engine node-tsc]  latest native speed relative to another engine
    measurements.py migrate OLD.sqlite          rebuild from the previous analytics tables
"""
import argparse
import json
from pathlib import Path
import sqlite3
import statistics
import sys

ROOT = Path(__file__).resolve().parents[1]
DATABASE = ROOT / 'measurements.sqlite'
RESULTS = ROOT / 'runtime' / 'results'
SCHEMA = '''CREATE TABLE IF NOT EXISTS measurements (
 workload TEXT NOT NULL, engine TEXT NOT NULL, history TEXT NOT NULL,
 PRIMARY KEY (workload, engine)) WITHOUT ROWID'''


def connect(path=DATABASE):
    db = sqlite3.connect(path)
    db.execute(SCHEMA)
    return db


def entry(at, durations, operations):
    """One history entry from a run's sample durations (ms) and operations per sample."""
    rates = [operations * 1000 / ms for ms in durations]
    return {'at': at, 'ms': round(statistics.median(durations), 6),
            'ops': round(statistics.fmean(rates), 3), 'samples': len(durations)}


def record(db, workload, engine, item):
    """Add `item` to the row's history unless that run is already there."""
    row = db.execute('SELECT history FROM measurements WHERE workload=? AND engine=?', (workload, engine)).fetchone()
    history = json.loads(row[0]) if row else []
    if any(old['at'] == item['at'] for old in history):
        return False
    history.append(item)
    history.sort(key=lambda old: old['at'])
    db.execute('INSERT OR REPLACE INTO measurements VALUES (?,?,?)', (workload, engine, json.dumps(history, separators=(',', ':'))))
    return True


def ingest(db, files):
    added = 0
    for file in files:
        data = json.loads(Path(file).read_text())
        execution = data.get('execution')
        if data.get('verifiedOnly') or not isinstance(execution, list) or not execution:
            continue
        workloads = {w['id']: w for w in data['manifest']['workloads']}
        samples = {}
        for row in execution:
            samples.setdefault((row['workload'], row['engine']), []).extend(row['durations'])
        for (workload, engine), durations in samples.items():
            definition = workloads.get(workload)
            if not definition or not durations:
                continue
            operations = definition['iterations'] * definition.get('operationsPerIteration', 1)
            added += record(db, workload, engine, entry(data['timestamp'], durations, operations))
    db.commit()
    return added


def migrate(db, old):
    """Rebuild histories from the previous analytics tables (execution runs only)."""
    source = sqlite3.connect(old)
    rows = source.execute('''SELECT r.timestamp, s.workload, s.engine, s.duration_ms, s.ops_per_second
        FROM analytics_samples s JOIN analytics_runs r USING (raw_sha)
        WHERE r.kind = 'execution' ORDER BY r.timestamp''')
    groups = {}
    for at, workload, engine, ms, ops in rows:
        group = groups.setdefault((at, workload, engine), ([], []))
        group[0].append(ms)
        group[1].append(ops)
    for (at, workload, engine), (durations, rates) in groups.items():
        record(db, workload, engine, {'at': at, 'ms': round(statistics.median(durations), 6),
                                      'ops': round(statistics.fmean(rates), 3), 'samples': len(durations)})
    db.commit()
    return len(groups)


def latest(db, engine):
    """Native speed over `engine` for each workload's most recent run that measured both."""
    histories = {}
    for workload, name, history in db.execute('SELECT workload, engine, history FROM measurements'):
        histories.setdefault(workload, {})[name] = {item['at']: item for item in json.loads(history)}
    rows = []
    for workload, engines in sorted(histories.items()):
        native, other = engines.get('native', {}), engines.get(engine, {})
        shared = sorted(set(native) & set(other))
        if shared:
            at = shared[-1]
            rows.append({'workload': workload, 'at': at, 'native': native[at]['ops'], engine: other[at]['ops'],
                         'ratio': native[at]['ops'] / other[at]['ops']})
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--database', type=Path, default=DATABASE)
    commands = parser.add_subparsers(dest='command', required=True)
    add = commands.add_parser('ingest')
    add.add_argument('files', nargs='*', type=Path)
    show = commands.add_parser('latest')
    show.add_argument('--engine', default='node-tsc')
    show.add_argument('--json', action='store_true')
    rebuild = commands.add_parser('migrate')
    rebuild.add_argument('old', type=Path)
    args = parser.parse_args()
    db = connect(args.database)
    if args.command == 'ingest':
        files = args.files or sorted(RESULTS.glob('*.json'))
        print(f'Added {ingest(db, files)} run entries from {len(files)} file(s).')
    elif args.command == 'migrate':
        print(f'Migrated {migrate(db, args.old)} run entries.')
    else:
        rows = latest(db, args.engine)
        if args.json:
            print(json.dumps(rows, indent=2))
        else:
            for row in rows:
                print(f"{row['workload']}: {row['ratio']:.3f}x ({row['at'][:10]}){'  review' if row['ratio'] < 1 else ''}")
    db.close()


if __name__ == '__main__':
    sys.exit(main())
