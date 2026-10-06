#!/usr/bin/env python3
"""Compact explicit closed-gate artifact directories; keep proof JSON loose."""
import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import tempfile
import zlib

CHUNK = 128 * 1024


def identity(stream):
    digest, size = hashlib.sha256(), 0
    while data := stream.read(CHUNK):
        digest.update(data)
        size += len(data)
    return {"sha256": digest.hexdigest(), "original_size": size}


def file_identity(path):
    with open(path, "rb") as source:
        return identity(source)


def save(path, value):
    # A fresh output directory makes each evidence filename immutable.
    with open(path, "x") as target:
        json.dump(value, target, indent=2)
        target.write("\n")
        target.flush()
        os.fsync(target.fileno())


def independent_blob(db, row):
    # Independent streamed decoder, separate from store.verify/export.
    blob_id, expected_hash, expected_size = row
    decoder, digest, size = zlib.decompressobj(), hashlib.sha256(), 0
    with db.blobopen("blobs", "compressed", blob_id, readonly=True) as blob:
        while chunk := blob.read(CHUNK):
            while chunk:
                data = decoder.decompress(chunk, CHUNK)
                size += len(data)
                if size > expected_size or decoder.unused_data:
                    raise ValueError("corrupt compressed record")
                digest.update(data)
                chunk = decoder.unconsumed_tail
    if not decoder.eof or size != expected_size or digest.hexdigest() != expected_hash:
        raise ValueError("archive byte identity mismatch")


def compact(store_path, root, database, plan_path, output):
    spec = importlib.util.spec_from_file_location("benchmark_store", store_path)
    store = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(store)
    root, database, output = Path(root).resolve(strict=True), Path(database).absolute(), Path(output).resolve(strict=False)
    plan_bytes = Path(plan_path).read_bytes()
    plan = json.loads(plan_bytes)
    # The caller must bind the actual completed runtime seal, after all consumers finish.
    proof = Path(plan["closedProof"]["path"])
    def closed_identity():
        if file_identity(proof)["sha256"] != plan["closedProof"]["sha256"]:
            raise ValueError("closed-gate proof drift")
    closed_identity()
    selections = plan["paths"]
    if not selections or len(set(selections)) != len(selections):
        raise ValueError("explicit unique artifact directories required")
    for selection in selections:
        parts = store.safe_parts(selection)
        if len(parts) != 3 or parts[:2] != ["runtime", "diagnostics"] or not parts[2].endswith("-artifacts"):
            raise ValueError("only direct diagnostic artifact directories may be compacted")
        os.close(store.directory_fd(root, parts))
        if output.is_relative_to(root / selection):
            raise ValueError("evidence output must stay outside selected artifact trees")
    paths = sorted(store.source_paths(root, selections))
    if not paths or len(paths) != len(set(paths)):
        raise ValueError("nonempty, nonoverlapping source paths required")
    original = {}
    allocated = 0
    for relative in paths:
        with store.open_source(root, relative) as source:
            original[relative] = identity(source)
            allocated += os.fstat(source.fileno()).st_blocks * 512
    output.mkdir()  # No accidental evidence overwrite.
    save(output / "plan.json", plan)
    save(output / "original-index.json", original)
    before_database = database.stat().st_size if database.exists() else 0
    before_allocated = database.stat().st_blocks * 512 if database.exists() else 0
    store.import_records(database, root, selections)
    store.verify(database)
    archived = {record["path"]: record for selection in selections
                for record in store.list_records(database, selection)}
    if set(original) != set(archived):
        raise ValueError("archive/original path sets differ")
    for relative in original:
        if any(archived[relative][key] != original[relative][key] for key in ("sha256", "original_size")):
            raise ValueError("archive/original byte identities differ: " + relative)
    with store.connect(database) as db:
        seen = set()
        for relative in paths:
            row = db.execute("SELECT id,blobs.sha256,original_size FROM records JOIN blobs USING(sha256) WHERE path=?", (relative,)).fetchone()
            if row[0] not in seen:
                independent_blob(db, row)
                seen.add(row[0])
    scratch = Path(tempfile.mkdtemp(prefix="tsrust-closed-gate-restore-"))
    removed = []
    try:
        for relative in paths:
            store.export_record(database, relative, scratch)
        restored = sorted(store.source_paths(scratch, selections))
        if restored != paths:
            raise ValueError("restored/original path sets differ")
        for relative in paths:
            with store.open_source(scratch, relative) as source:
                if identity(source) != original[relative]:
                    raise ValueError("restored byte identity differs: " + relative)
            with store.open_source(root, relative) as source:
                if identity(source) != original[relative]:
                    raise ValueError("original drift before removal: " + relative)
        closed_identity()
        save(output / "verified-index.json", archived)
        save(output / "verification.json", {"exactPathsAndBytes": len(paths), "independentBlobChecks": len(seen), "closedProof": plan["closedProof"]})
        for relative in paths:
            parts = store.safe_parts(relative)
            parent = store.directory_fd(root, parts[:-1])
            try:
                with store.open_source(root, relative) as source:
                    if identity(source) != original[relative]:
                        raise ValueError("original drift at removal: " + relative)
                os.unlink(parts[-1], dir_fd=parent)
                removed.append(relative)
            finally:
                os.close(parent)
        # Remove only empty directories beneath the explicitly selected trees.
        for selection in selections:
            for directory, _, _ in os.walk(root / selection, topdown=False):
                try:
                    os.rmdir(directory)
                except OSError:
                    pass
        store.verify(database)
        closed_identity()
        save(output / "migration.json", {"kind": "executed-closed-gate-artifact-compaction", "timestamp": datetime.now(timezone.utc).isoformat(), "planSHA256": hashlib.sha256(plan_bytes).hexdigest(), "selection": selections, "files": len(paths), "originalLogicalBytes": sum(x["original_size"] for x in original.values()), "originalAllocatedBytes": allocated, "database": {"path": str(database), "sha256": file_identity(database)["sha256"], "logicalBytes": database.stat().st_size, "allocatedBytes": database.stat().st_blocks * 512}, "databaseGrowthBytes": database.stat().st_size - before_database, "grossAllocatedBytesReclaimed": allocated - (database.stat().st_blocks * 512 - before_allocated), "verification": "original/restored/archive paths, sizes and SHA256; independent zlib; rehash each leaf before unlink", "rollbackOnFailure": True})
    except BaseException:
        # Keep verified scratch evidence until all removed originals are restored.
        for relative in removed:
            destination = root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            if destination.exists():
                if file_identity(destination) != original[relative]:
                    raise ValueError("rollback destination conflict; scratch retained: " + str(scratch))
            else:
                with store.open_source(scratch, relative) as source, destination.open("xb") as target:
                    shutil.copyfileobj(source, target, CHUNK)
        raise
    else:
        shutil.rmtree(scratch)
    print(json.dumps({"files": len(paths), "manifest": str(output / "migration.json")}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for argument in ("store", "root", "database", "plan", "output"):
        parser.add_argument("--" + argument, required=True)
    args = parser.parse_args()
    compact(args.store, args.root, args.database, args.plan, args.output)
