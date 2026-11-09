#!/usr/bin/env python3
"""Run every `tsrust test` suite with one compiler build.

Each directory's `.test.tsr` files compile together into one native program.
Suites come from the repository (fixtures and examples) and from the generated
catalog suites. A failing suite fails the run; all suites still run so one CI
pass reports every failure.

usage: run-suites.py TSRUST [GENERATED_CATALOG_DIRECTORY]
"""
import json
import os
import subprocess
import sys
import tempfile
import time
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Fixtures that intentionally fail; their Rust host tests assert the failure report.
EXPECTED_FAILURES = {"crates/tsrust-compiler/tests/fixtures/testing-async/failures.test.tsr"}



def suites(catalog):
    groups = defaultdict(list)
    tracked = subprocess.run(["git", "ls-files", "*.test.tsr"], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", check=True).stdout.split()
    for file in tracked:
        if file not in EXPECTED_FAILURES:
            groups[str(Path(file).parent)].append(file)
    if catalog:
        for directory in sorted(Path(catalog).iterdir()):
            if directory.is_dir():
                groups[str(directory)] = sorted(str(path) for path in directory.glob("*.test.tsr"))
    return dict(sorted(groups.items()))


def main():
    compiler = str(Path(sys.argv[1]).resolve())
    catalog = sys.argv[2] if len(sys.argv) > 2 else None
    results, failed = [], 0
    env = {**os.environ, "TSRUST_CACHE_DIR": os.environ.get("TSRUST_CACHE_DIR", str(ROOT / "target/tsrust-native")),
           "TSRUST_REALTIME_TLS_DIR": str(ROOT / "crates/tsrust-compiler/tests/fixtures/tls")}
    for directory, files in suites(catalog).items():
        started = time.monotonic()
        # Sources run in place (relative imports resolve from each file); a fresh
        # working directory keeps files the suite writes out of the tree.
        # Rust-package adapters resolve their locked crate closure first.
        for bindings in sorted((ROOT / directory).glob("*.bindings.json")):
            subprocess.run([compiler, "bindings", "resolve", str(bindings)], cwd=ROOT, env=env, stdin=subprocess.DEVNULL, check=True)
        try:
            with tempfile.TemporaryDirectory(prefix="tsrust-suite-") as working:
                process = subprocess.run([compiler, "test", *[str((ROOT / file).resolve()) for file in files], "--color", "never"],
                                         cwd=working, env=env, stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=600)
        except subprocess.TimeoutExpired as error:
            process = subprocess.CompletedProcess(error.cmd, 124, error.stdout or "", f"suite exceeded 600s\n{error.stderr or ''}")
        seconds = round(time.monotonic() - started, 1)
        ok = process.returncode == 0
        failed += not ok
        results.append({"suite": directory, "files": len(files), "seconds": seconds, "ok": ok})
        print(f"{'PASS' if ok else 'FAIL'} {directory} ({len(files)} files, {seconds}s)", flush=True)
        if not ok:
            print(str(process.stdout)[-4000:] + str(process.stderr)[-4000:], flush=True)
    print(json.dumps({"suites": len(results), "failed": failed, "seconds": round(sum(row["seconds"] for row in results), 1)}))
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
