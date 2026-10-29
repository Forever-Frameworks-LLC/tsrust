#!/usr/bin/env python3
"""Export accepted, indexed measurements without logs or identifying metadata."""
import argparse
from contextlib import closing
import datetime
import json
import os
from pathlib import Path
import re
import sqlite3
import tempfile

import analytics
import store

APPLICATION_ID = 0x5453424D
TABLES = ('analytics_runs', 'analytics_workloads', 'analytics_samples',
          'analytics_summaries', 'analytics_profiles')
HASH = re.compile(r'[0-9a-f]{64}')
IDENTIFIER = re.compile(r'[a-zA-Z0-9_-]{1,160}')
VERSION = re.compile(r'v?\d+(?:\.\d+){0,3}(?:[-+][a-zA-Z0-9.-]+)?')
VERSION_KEYS = ('node', 'rust', 'bun', 'deno', 'typescript', 'ts-node', 'tsx',
                'bigdecimal', 'chrono', 'chronoTz', 'dateFns', 'decimalJs',
                'getrandom', 'rusqlite', 'sqliteNative', 'sqliteNode', 'uuid')
COMPONENT_KEYS = ('acorn', 'ada', 'amaro', 'ares', 'brotli', 'cjs_module_lexer',
                  'cldr', 'icu', 'llhttp', 'modules', 'napi', 'nbytes', 'ncrypto',
                  'nghttp2', 'node', 'openssl', 'simdjson', 'simdutf', 'sqlite',
                  'tz', 'undici', 'unicode', 'uv', 'uvwasi', 'v8', 'zlib', 'zstd')
SCOPES = {'direct-rust-reference', 'execution-only',
          'fresh-process compilation; forward acyclic scalar function dependencies only',
          'in-process-128-insert-transaction', 'in-process-matching',
          'in-process-worker-lifecycle', 'native-allocation-profile',
          'runtime-execution', 'runtime-startup-compilation',
          'loopback JSON/schema and contextual streaming requests; excludes startup/build',
          'verified-https-http2-request-batch'}


def digest(value):
    if not isinstance(value, str) or not HASH.fullmatch(value):
        raise ValueError('invalid measurement SHA256')
    return value


def identifier(value):
    if not isinstance(value, str) or not IDENTIFIER.fullmatch(value):
        raise ValueError('invalid measurement identifier')
    return value


def version(value):
    if not isinstance(value, str):
        return None
    if VERSION.fullmatch(value):
        return value
    rust = re.fullmatch(r'rustc (\d+\.\d+\.\d+) \(([0-9a-f]+) (\d{4}-\d\d-\d\d)\)', value)
    if rust:
        return value
    deno = re.match(r'deno (\d+\.\d+\.\d+) \(', value)
    return deno[1] if deno else None


def metadata(value):
    """Construct a public record; never copy arbitrary objects or free text."""
    out = {}
    machine = value.get('machine', {})
    hardware = {}
    for key, allowed in [('platform', {'darwin', 'linux', 'win32', 'freebsd'}),
                         ('architecture', {'arm64', 'aarch64', 'x64', 'x86_64', 'arm', 'ia32'})]:
        if machine.get(key) in allowed:
            hardware[key] = machine[key]
    cpu = machine.get('cpu', '')
    if isinstance(cpu, str) and re.fullmatch(r'Apple M\d{1,2}(?: Pro| Max| Ultra)?', cpu):
        hardware['cpu'] = cpu
    for key in ['logicalCPUs', 'logicalCpus', 'availableParallelism', 'totalMemoryBytes']:
        if type(machine.get(key)) is int and 0 < machine[key] <= 2**50:
            hardware[key] = machine[key]
    if hardware:
        out['machine'] = hardware
    for group, keys in [('versions', VERSION_KEYS), ('nodeComponents', COMPONENT_KEYS)]:
        versions = {key: result for key in keys
                    if (result := version(value.get(group, {}).get(key))) is not None}
        # IANA tzdata uses year + revision rather than a package version.
        tz = value.get(group, {}).get('tz')
        if group == 'nodeComponents' and isinstance(tz, str) and re.fullmatch(r'\d{4}[a-z]', tz):
            versions['tz'] = tz
        if versions:
            out[group] = versions
    if result := version(value.get('rustVersion')):
        out['rustVersion'] = result
    for key in ['measurementPlanSha256', 'harnessSha256', 'adapterSha256', 'lockSha256',
                'releaseSha256', 'releaseRuntimeSourceSha256', 'nativeLockSha256']:
        if value.get(key) is not None:
            out[key] = digest(value[key])
    native_lock = value.get('versions', {}).get('nativeLockSha256')
    if native_lock is not None:
        out['nativeLockSha256'] = digest(native_lock)
    for key in ['releaseSourceCommit', 'releaseBaseCommit']:
        if key in value:
            if not isinstance(value[key], str) or not re.fullmatch(r'[0-9a-f]{40}', value[key]):
                raise ValueError('invalid source commit')
            out[key] = value[key]
    for key in ['compilerInputs', 'oracleInputs', 'numberAdapterInputs', 'sliceAdapterInputs']:
        inputs = value.get(key)
        if isinstance(inputs, list):
            out[key] = [digest(item[1] if isinstance(item, list) else item) for item in inputs]
        elif isinstance(inputs, dict) and 'sha256' in inputs:
            out[key] = {'sha256': digest(inputs['sha256'])}
            commit = inputs.get('sourceCommit')
            if isinstance(commit, str) and re.fullmatch(r'[0-9a-f]{40}', commit):
                out[key]['sourceCommit'] = commit
            if type(inputs.get('sourceTreeDirty')) is bool:
                out[key]['sourceTreeDirty'] = inputs['sourceTreeDirty']
    if isinstance(value.get('release'), dict):
        for key in ['sha256', 'runtimeSourceSha256']:
            if key in value['release']:
                out['release' + key[0].upper() + key[1:]] = digest(value['release'][key])
        for key in ['sourceCommit', 'baseCommit']:
            commit = value['release'].get(key)
            if isinstance(commit, str) and re.fullmatch(r'[0-9a-f]{40}', commit):
                out['release' + key[0].upper() + key[1:]] = commit
    elif type(value.get('release')) is bool:
        out['release'] = value['release']
    if 'measuredEngines' in value:
        out['measuredEngines'] = [identifier(item) for item in value['measuredEngines']]
    return out


def context(value):
    out = {}
    for key in ['emittedSha256', 'referenceSourceSha256', 'matchedNativeSourceSha256']:
        if value.get(key) is not None:
            out[key] = digest(value[key])
    # Historical prose and result strings are not needed to query measurements.
    if value.get('parameters') in (['int'], ['string'], ['int', 'int'], ['int', 'int', 'string']):
        out['parameters'] = value['parameters']
    for key in ['functionCount', 'expectedChecksum']:
        if type(value.get(key)) is int:
            out[key] = value[key]
    return out


def connect(database):
    path = Path(database).absolute()
    if path.is_symlink():
        raise ValueError('database must not be a symlink')
    db = sqlite3.connect(path.as_uri() + '?mode=ro', uri=True, isolation_level=None)
    if (db.execute('PRAGMA application_id').fetchone()[0] != APPLICATION_ID or
            db.execute('PRAGMA user_version').fetchone()[0] != 1):
        db.close()
        raise ValueError('unsupported public measurement database')
    return db


def validate(db):
    names = {row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
    if names != set(TABLES):
        raise ValueError('unexpected public database tables')
    if db.execute('PRAGMA integrity_check').fetchall() != [('ok',)] or db.execute('PRAGMA foreign_key_check').fetchall():
        raise ValueError('invalid measurement database')
    if db.execute('PRAGMA freelist_count').fetchone()[0] != 0:
        raise ValueError('public database must have no deleted pages')
    for raw, proof, kind, timestamp, epoch, scope, data in db.execute('SELECT * FROM analytics_runs'):
        digest(raw); digest(proof); identifier(kind); identifier(epoch)
        if not re.fullmatch(r'\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?(?:Z|[+-]\d\d:\d\d)', timestamp):
            raise ValueError('invalid measurement timestamp')
        datetime.datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
        if scope not in SCOPES or metadata(json.loads(data)) != json.loads(data):
            raise ValueError('unapproved public run metadata or scope')
    for raw, name, source, counterpart, iterations, operations, data in db.execute('SELECT * FROM analytics_workloads'):
        digest(raw); identifier(name); digest(source)
        if counterpart is not None:
            digest(counterpart)
        if type(iterations) is not int or iterations <= 0 or type(operations) is not int or operations <= 0:
            raise ValueError('invalid operation count')
        if context(json.loads(data)) != json.loads(data):
            raise ValueError('unapproved public workload metadata')
    for table in ['analytics_samples', 'analytics_summaries']:
        for (engine,) in db.execute(f'SELECT DISTINCT engine FROM {table}'):
            identifier(engine)
    return {table: db.execute(f'SELECT count(*) FROM {table}').fetchone()[0] for table in TABLES}


def export(database, destination):
    destination = Path(destination).absolute()
    if destination.exists() or destination.is_symlink():
        raise FileExistsError('refusing to overwrite public measurements')
    source = store.connect(database)
    temporary = None
    try:
        source.execute('BEGIN')
        if source.execute('PRAGMA quick_check').fetchall() != [('ok',)] or source.execute('PRAGMA foreign_key_check').fetchall():
            raise ValueError('invalid source database')
        with tempfile.NamedTemporaryFile(dir=destination.parent, suffix='.sqlite', delete=False) as file:
            temporary = Path(file.name)
        with closing(sqlite3.connect(temporary)) as target:
            target.execute('PRAGMA foreign_keys=ON')
            # Same query schema, with original-record hashes retained as identities only.
            # WITHOUT ROWID stores rows in their composite primary key instead of a
            # second copy in an autoindex, roughly halving the published file.
            schema = analytics.SCHEMA.replace(' REFERENCES blobs(sha256)', '')
            schema = re.sub(r'(CREATE TABLE IF NOT EXISTS analytics_(?:workloads|samples|summaries|profiles) \((?:[^;]|\n)*?\))\s*;', r'\1 WITHOUT ROWID;', schema)
            target.executescript(schema)
            target.execute(f'PRAGMA application_id={APPLICATION_ID}')
            target.execute('PRAGMA user_version=1')
            for table in TABLES:
                for original in source.execute(f'SELECT * FROM {table} ORDER BY 1,2,3'):
                    row = list(original)
                    if table == 'analytics_runs':
                        row[-1] = analytics.encode(metadata(json.loads(row[-1])))
                    elif table == 'analytics_workloads':
                        row[-1] = analytics.encode(context(json.loads(row[-1])))
                    target.execute(f'INSERT INTO {table} VALUES({",".join("?" for _ in row)})', row)
                # Every numeric value and sample boundary must survive unchanged.
                columns = [r[1] for r in source.execute(f'PRAGMA table_info({table})')
                           if r[1] not in ('metadata', 'context')]
                query = f'SELECT {",".join(columns)} FROM {table} ORDER BY 1,2,3'
                if any(a != b for a, b in zip(source.execute(query), target.execute(query), strict=True)):
                    raise ValueError('measurement identity changed during export')
            counts = validate(target)
            if counts['analytics_runs'] == 0:
                raise ValueError('no indexed measurements to export')
            target.commit()
        with closing(connect(temporary)) as target:
            validate(target)
        # Atomic, no-overwrite publication after closing both SQLite writers.
        os.link(temporary, destination)
        return {'counts': counts, 'sha256': analytics.sha(destination), 'bytes': destination.stat().st_size}
    finally:
        source.close()
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('database')
    parser.add_argument('destination')
    args = parser.parse_args()
    print(json.dumps(export(args.database, args.destination), indent=2))


if __name__ == '__main__':
    main()
