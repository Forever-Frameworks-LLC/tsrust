#!/usr/bin/env python3
"""Exact-byte, append-only benchmark archive. Requires Python 3.11+."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import sqlite3
import stat
import sys
import tempfile
from urllib.parse import quote
import zlib

CHUNK = 128 * 1024
APPLICATION_ID = 0x54534253
SCHEMA = """
CREATE TABLE blobs (
    id INTEGER PRIMARY KEY,
    sha256 TEXT NOT NULL UNIQUE,
    original_size INTEGER NOT NULL CHECK(original_size >= 0),
    compressed BLOB NOT NULL
);
CREATE TABLE records (
    path TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    sha256 TEXT NOT NULL REFERENCES blobs(sha256)
);
CREATE TRIGGER immutable_blobs_update BEFORE UPDATE ON blobs
BEGIN SELECT RAISE(ABORT, 'immutable blob'); END;
CREATE TRIGGER immutable_blobs_delete BEFORE DELETE ON blobs
BEGIN SELECT RAISE(ABORT, 'immutable blob'); END;
CREATE TRIGGER immutable_records_update BEFORE UPDATE ON records
BEGIN SELECT RAISE(ABORT, 'immutable path binding'); END;
CREATE TRIGGER immutable_records_delete BEFORE DELETE ON records
BEGIN SELECT RAISE(ABORT, 'immutable path binding'); END;
"""


def safe_parts(relative):
    if not relative or '\\' in relative or '\x00' in relative:
        raise ValueError('invalid relative path')
    parts = relative.split('/')
    if PurePosixPath(relative).is_absolute() or any(p in ('', '.', '..') for p in parts):
        raise ValueError('path must be relative without traversal')
    return parts


def directory_fd(root, parts, create=False):
    """Walk from a fixed root, refusing symlinks even under concurrent renames."""
    fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in parts:
            if create:
                try:
                    os.mkdir(part, dir_fd=fd)
                except FileExistsError:
                    pass
            next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        return fd
    except BaseException:
        os.close(fd)
        raise


def open_source(root, relative):
    parts = safe_parts(relative)
    parent = directory_fd(root, parts[:-1])
    try:
        fd = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
    finally:
        os.close(parent)
    if not stat.S_ISREG(os.fstat(fd).st_mode):
        os.close(fd)
        raise ValueError('source must be a regular file: ' + relative)
    return os.fdopen(fd, 'rb')


def connect(database, writable=False):
    path = Path(database).absolute()
    if path.is_symlink():
        raise ValueError('database must not be a symlink')
    if writable:
        db = sqlite3.connect(path, isolation_level=None)
    else:
        db = sqlite3.connect('file:' + quote(str(path), safe='/') + '?mode=ro', uri=True,
                             isolation_level=None)
    try:
        db.execute('PRAGMA foreign_keys=ON')
        version = db.execute('PRAGMA user_version').fetchone()[0]
        app = db.execute('PRAGMA application_id').fetchone()[0]
        if writable and version == 0 and app == 0:
            if db.execute("SELECT 1 FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'").fetchone():
                raise ValueError('refusing unrelated database')
            db.executescript('BEGIN IMMEDIATE;\n' + SCHEMA +
                             f'PRAGMA application_id={APPLICATION_ID};\n'
                             'PRAGMA user_version=1;\nCOMMIT;')
        elif version != 1 or app != APPLICATION_ID:
            raise ValueError('unsupported benchmark store')
        return db
    except BaseException:
        db.close()
        raise


def decode_blob(db, row, sink=None):
    blob_id, digest, expected_size = row
    decoder = zlib.decompressobj()
    checksum = hashlib.sha256()
    size = 0
    with db.blobopen('blobs', 'compressed', blob_id, readonly=True) as blob:
        while data := blob.read(CHUNK):
            while data:
                decoded = decoder.decompress(data, CHUNK)
                size += len(decoded)
                if size > expected_size:
                    raise ValueError('corrupt blob: oversized output ' + digest)
                checksum.update(decoded)
                if sink is not None:
                    sink.write(decoded)
                if decoder.unused_data:
                    raise ValueError('corrupt blob: trailing compressed data ' + digest)
                data = decoder.unconsumed_tail
    if not decoder.eof or size != expected_size or checksum.hexdigest() != digest:
        raise ValueError('corrupt blob: original byte identity mismatch ' + digest)
    return size


def source_paths(root, selections):
    for relative in selections:
        parts = safe_parts(relative)
        parent = directory_fd(root, parts[:-1])
        try:
            mode = os.stat(parts[-1], dir_fd=parent, follow_symlinks=False).st_mode
        finally:
            os.close(parent)
        selected = root.joinpath(*parts)
        if stat.S_ISLNK(mode):
            raise ValueError('source symlinks are unsupported: ' + relative)
        if stat.S_ISDIR(mode):
            for directory, dirs, files in os.walk(selected, followlinks=False):
                dirs.sort()
                for name in dirs + files:
                    candidate = Path(directory, name)
                    if candidate.is_symlink():
                        raise ValueError('source symlinks are unsupported: ' + str(candidate))
                for name in sorted(files):
                    yield Path(directory, name).relative_to(root).as_posix()
        else:
            yield relative


def record_kind(relative):
    parts = safe_parts(relative)
    for folder, kind in [('results', 'result'), ('diagnostics', 'diagnostic'),
                         ('approximate-references', 'reference')]:
        if folder in parts:
            return kind
    return 'artifact'


def import_records(database, root, selections, kind=None):
    root = Path(root).resolve(strict=True)
    db = connect(database, writable=True)
    count = 0
    try:
        db.execute('BEGIN IMMEDIATE')
        for relative in source_paths(root, selections):
            source_path = root.joinpath(*safe_parts(relative))
            database_path = Path(database).resolve()
            if str(source_path) in {str(database_path) + suffix for suffix in ('', '-journal', '-wal', '-shm')}:
                raise ValueError('cannot import the archive database or its sidecars')
            with open_source(root, relative) as source, tempfile.TemporaryFile() as compressed:
                before = os.fstat(source.fileno())
                checksum = hashlib.sha256()
                compressor = zlib.compressobj(6)
                size = 0
                while data := source.read(CHUNK):
                    checksum.update(data)
                    size += len(data)
                    compressed.write(compressor.compress(data))
                compressed.write(compressor.flush())
                after = os.fstat(source.fileno())
                if (before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
                        after.st_size, after.st_mtime_ns, after.st_ctime_ns) or size != after.st_size:
                    raise ValueError('source changed during import: ' + relative)
                digest = checksum.hexdigest()
                category = kind or record_kind(relative)
                existing = db.execute('SELECT sha256, kind FROM records WHERE path=?', (relative,)).fetchone()
                if existing and existing != (digest, category):
                    raise ValueError('immutable path conflict: ' + relative)
                blob = db.execute('SELECT id, sha256, original_size FROM blobs WHERE sha256=?',
                                  (digest,)).fetchone()
                if blob:
                    if blob[2] != size:
                        raise ValueError('corrupt blob size: ' + digest)
                    decode_blob(db, blob)
                else:
                    cursor = db.execute('INSERT INTO blobs(sha256,original_size,compressed) VALUES(?,?,zeroblob(?))',
                                        (digest, size, compressed.tell()))
                    compressed.seek(0)
                    with db.blobopen('blobs', 'compressed', cursor.lastrowid) as target:
                        while data := compressed.read(CHUNK):
                            target.write(data)
                if not existing:
                    db.execute('INSERT INTO records(path,kind,sha256) VALUES(?,?,?)',
                               (relative, category, digest))
                count += 1
        db.execute('COMMIT')
        return count
    except BaseException:
        if db.in_transaction:
            db.execute('ROLLBACK')
        raise
    finally:
        db.close()


def verify(database):
    db = connect(database)
    try:
        for (result,) in db.execute('PRAGMA quick_check'):
            if result != 'ok':
                raise ValueError('SQLite integrity failure: ' + result)
        if db.execute('PRAGMA foreign_key_check').fetchone():
            raise ValueError('dangling record binding')
        for relative, kind in db.execute('SELECT path,kind FROM records'):
            safe_parts(relative)
            if not kind:
                raise ValueError('empty record kind')
        count = 0
        for row in db.execute('SELECT id,sha256,original_size FROM blobs'):
            decode_blob(db, row)
            count += 1
        return count
    finally:
        db.close()


def list_records(database, prefix=None):
    if prefix is not None:
        safe_parts(prefix)
    db = connect(database)
    try:
        where = '' if prefix is None else ' WHERE path=? OR substr(path,1,?)=?'
        parameters = () if prefix is None else (prefix, len(prefix) + 1, prefix + '/')
        for path, kind, digest, size, compressed_size in db.execute(
                'SELECT path,kind,records.sha256,original_size,length(compressed) '
                'FROM records JOIN blobs USING(sha256)' + where + ' ORDER BY path', parameters):
            safe_parts(path)
            yield dict(path=path, kind=kind, sha256=digest, original_size=size,
                       compressed_size=compressed_size)
    finally:
        db.close()


def export_record(database, relative, root):
    parts = safe_parts(relative)
    root = Path(root).resolve(strict=True)
    db = connect(database)
    parent = None
    temporary = None
    try:
        row = db.execute('SELECT id,blobs.sha256,original_size FROM records JOIN blobs USING(sha256) '
                         'WHERE path=?', (relative,)).fetchone()
        if row is None:
            raise ValueError('unknown record: ' + relative)
        parent = directory_fd(root, parts[:-1], create=True)
        # Stage in the target directory; publish with link(), which cannot overwrite.
        candidate = '.benchmark-export-' + os.urandom(16).hex()
        fd = os.open(candidate, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                     0o600, dir_fd=parent)
        temporary = candidate
        with os.fdopen(fd, 'wb') as target:
            decode_blob(db, row, target)
            target.flush()
            os.fsync(target.fileno())
        os.link(temporary, parts[-1], src_dir_fd=parent, dst_dir_fd=parent,
                follow_symlinks=False)
        os.unlink(temporary, dir_fd=parent)
        temporary = None
        os.fsync(parent)
    finally:
        if temporary is not None:
            os.unlink(temporary, dir_fd=parent)
        if parent is not None:
            os.close(parent)
        db.close()


def read_record(database, relative, sink):
    """Verify before emitting any original bytes to the report reader bridge."""
    safe_parts(relative)
    db = connect(database)
    try:
        row = db.execute('SELECT id,blobs.sha256,original_size FROM records JOIN blobs USING(sha256) '
                         'WHERE path=?', (relative,)).fetchone()
        if row is None:
            raise ValueError('unknown record: ' + relative)
        with tempfile.TemporaryFile() as original:
            decode_blob(db, row, original)
            original.seek(0)
            shutil.copyfileobj(original, sink, CHUNK)
    finally:
        db.close()


def restore_tree(database, root, prefix=None):
    """Restore original paths using individually atomic, no-overwrite exports."""
    root = Path(root).resolve(strict=True)
    os.close(directory_fd(root, []))
    verify(database)
    count = 0
    for record in list_records(database, prefix):
        export_record(database, record['path'], root)
        count += 1
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    load = commands.add_parser('import', help='atomically import files/directories relative to --root')
    load.add_argument('database')
    load.add_argument('paths', nargs='+')
    load.add_argument('--root', required=True)
    load.add_argument('--kind')
    commands.add_parser('verify').add_argument('database')
    listing = commands.add_parser('list')
    listing.add_argument('database')
    listing.add_argument('--prefix')
    export = commands.add_parser('export', help='atomically export one record under an existing --root')
    export.add_argument('database')
    export.add_argument('path')
    export.add_argument('--root', required=True)
    read = commands.add_parser('read', help='emit verified exact record bytes on stdout')
    read.add_argument('database')
    read.add_argument('path')
    restore = commands.add_parser('restore', help='restore original relative paths; never overwrite')
    restore.add_argument('database')
    restore.add_argument('--root', required=True)
    restore.add_argument('--prefix')
    args = parser.parse_args()
    try:
        if args.command == 'import':
            print(f'Imported {import_records(args.database, args.root, args.paths, args.kind)} records')
        elif args.command == 'verify':
            print(f'Verified {verify(args.database)} unique blobs')
        elif args.command == 'list':
            for record in list_records(args.database, args.prefix):
                print(json.dumps(record, ensure_ascii=False))
        elif args.command == 'export':
            export_record(args.database, args.path, args.root)
            print('Exported ' + args.path)
        elif args.command == 'read':
            read_record(args.database, args.path, sys.stdout.buffer)
        else:
            print(f'Restored {restore_tree(args.database, args.root, args.prefix)} records')
    except (ValueError, OSError, sqlite3.Error, zlib.error) as error:
        parser.exit(1, f'error: {error}\n')


if __name__ == '__main__':
    main()
