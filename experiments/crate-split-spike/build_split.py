#!/usr/bin/env python3
"""Build crates written by split_modules.py with rustc, in dependency order.

usage: build_split.py SPLIT_DIR RUNTIME_ARTIFACTS_DIR OUTPUT_BINARY [--jobs N]

Uses the same flags as a `tsrust dev` build (edition 2024, opt-level 0,
overflow checks, incremental per crate). Prints per-crate and total seconds.
"""
import concurrent.futures
import json
import os
import subprocess
import sys
import time

split, artifacts, output = sys.argv[1:4]
jobs = int(sys.argv[sys.argv.index("--jobs") + 1]) if "--jobs" in sys.argv else os.cpu_count()
manifest = json.load(open(os.path.join(split, "crates.json")))
runtime = os.path.join(artifacts, "libtsrust_runtime.rlib")
libs = os.path.join(split, "lib")
os.makedirs(libs, exist_ok=True)
# The staged runtime rlib carries a hashed crate name; dependents of crates built
# against it look it up by that name, so expose it under that file name too.
import re as _re
_name = _re.search(rb"tsrust_runtime_[0-9a-f]{16}", open(runtime, "rb").read())
if _name:
    alias = os.path.join(libs, "lib" + _name.group(0).decode() + ".rlib")
    if not os.path.exists(alias):
        os.symlink(runtime, alias)
toolchain = os.environ.get("TSRUST_RUST_TOOLCHAIN", "1.99.1")


def rustc(crate):
    name = crate["name"]
    program = name == "program"
    args = ["rustup", "run", toolchain, "rustc", "-L", f"dependency={artifacts}", "-L", f"dependency={libs}",
            "--extern", f"tsrust_runtime={runtime}", "--cap-lints", "allow", "--edition=2024",
            "-C", "opt-level=0", "-C", "overflow-checks=on",
            "-C", f"incremental={os.path.join(split, 'incremental', name)}"]
    for dependency in crate["dependencies"]:
        args += ["--extern", f"{dependency}={os.path.join(libs, 'lib' + dependency + '.rlib')}"]
    if program:
        args += [os.path.join(split, f"{name}.rs"), "-o", output]
    else:
        args += ["--crate-type", "rlib", "--crate-name", name, os.path.join(split, f"{name}.rs"), "-o", os.path.join(libs, f"lib{name}.rlib")]
    started = time.time()
    result = subprocess.run(args, capture_output=True, text=True)
    return name, time.time() - started, result


def report(name, seconds, result):
    status = "ok" if result.returncode == 0 else "FAILED"
    print(f"{seconds:7.2f}s {status} {name}", flush=True)
    if result.returncode:
        sys.stderr.write(result.stderr[-6000:])
        return False
    return True


started = time.time()
only = set(sys.argv[sys.argv.index("--only") + 1].split(",")) if "--only" in sys.argv else None
types = [crate for crate in manifest if crate["name"] == "tsrust_types" and (only is None or "tsrust_types" in only)]
modules = [crate for crate in manifest if crate["name"] not in ("tsrust_types", "program") and (only is None or crate["name"] in only)]
program = [crate for crate in manifest if crate["name"] == "program"]
ok = all(report(*rustc(crate)) for crate in types)
if ok:
    with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
        results = list(pool.map(rustc, modules))
    ok = all([report(*result) for result in sorted(results, key=lambda r: -r[1])])
if ok:
    ok = all(report(*rustc(crate)) for crate in program)
print(f"total {time.time() - started:.2f}s {'ok' if ok else 'FAILED'}")
sys.exit(0 if ok else 1)
