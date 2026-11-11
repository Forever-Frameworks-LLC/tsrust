"""Prepared B03 admission driver. Run only under the integrator's build/timing lease.

This is a direct-Rust prototype, not a public implementation or a registered
native/Node workload. Output contains numeric observations and source identities;
environment, directory, executable and hostname values are never recorded.
Compile durations use fresh output files and the existing dependency cache.
Binary sizes describe this admission harness, not per-API product overhead.
Allocation counters cover Rust allocator calls; OS allocations are not inferred.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
RUST = ["rustup", "run", "1.99.1", "rustc"]
CARGO = ["rustup", "run", "1.99.1", "cargo"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command, env):
    result = subprocess.run(command, env=env, text=True, capture_output=True)
    if result.returncode:
        # Diagnostics may identify source locations. No measured OS value is
        # printed by the programs, including their failure controls.
        raise RuntimeError(result.stderr or result.stdout or "child failed")
    return result.stdout


def quiet_guard():
    # The owner requires quiet timing below load 4. The outer watchdog bounds
    # waiting and owns the process group; this is not a public runtime limit.
    if hasattr(os, "getloadavg"):
        while os.getloadavg()[0] >= 4:
            time.sleep(5)


def fixtures():
    values = {
        "TSRUST_B03_PROFILE_FIXTURES": "1",
        "TSRUST_B03_EMPTY": "",
        "TSRUST_B03_TEXT": "héllo 🦀",
        "PATH": "/tsrust-b03-path",
    }
    if os.name == "nt":
        values.update(USERPROFILE=r"C:\tsrust-b03-home", TEMP=r"C:\tsrust-b03-temp", TMP=r"C:\tsrust-b03-temp")
    else:
        values.update(HOME="/tsrust-b03-home", TMPDIR="/tsrust-b03-temp")
    return values


def compiler_environment(args):
    values = os.environ.copy()
    for key in tuple(values):
        if key in {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "CARGO_INCREMENTAL"} or key.startswith(("RUSTC_", "CARGO_PROFILE_", "CARGO_BUILD_", "CARGO_TARGET_")):
            del values[key]
    values.update(CARGO_HOME=str(args.cargo_home), CARGO_TARGET_DIR=str(args.target_dir), CARGO_INCREMENTAL="0", CARGO_BUILD_JOBS="1")
    return values


def hostname_dependency(args, env):
    if args.hostname_manifest is None:
        return [], None
    command = CARGO + ["build", "--offline", "--locked", "--release", "--lib", "--message-format=json", "--manifest-path", str(args.hostname_manifest)]
    started = time.perf_counter_ns()
    output = run(command, env)
    preparation_ns = time.perf_counter_ns() - started
    libraries = set()
    for line in output.splitlines():
        row = json.loads(line)
        if row.get("reason") != "compiler-artifact" or row["target"]["name"] != "hostname":
            continue
        for name in row["filenames"]:
            path = Path(name).resolve()
            path.relative_to(args.target_dir)
            if path.suffix == ".rlib":
                libraries.add(path)
    if len(libraries) != 1:
        raise RuntimeError("expected one exact hostname compiler artifact")
    library = libraries.pop()
    return ["--cfg", "hostname_admission", "--extern", f"hostname={library}", "-L", f"dependency={library.parent}"], {
        "manifest_sha256": digest(args.hostname_manifest),
        "lock_sha256": digest(args.hostname_manifest.with_name("Cargo.lock")),
        "library_sha256": digest(library),
        "untimed_dependency_preparation_ns": preparation_ns,
    }


def compile_reference(args, env, name, flags):
    binary = args.output_dir / (name + (".exe" if os.name == "nt" else ""))
    if binary.exists():
        raise RuntimeError("fresh admission output path required")
    command = RUST + ["--edition=2024", "-O", str(HERE / "reference.rs"), "-o", str(binary)] + flags
    started = time.perf_counter_ns()
    run(command, env)
    return binary, {"name": name, "compile_ns": time.perf_counter_ns() - started, "binary_bytes": binary.stat().st_size, "binary_sha256": digest(binary)}


def rust_rows(output, mode):
    rows = []
    for line in output.splitlines():
        fields = line.split("\t")
        if fields[0] != mode:
            raise RuntimeError("unexpected reference output")
        expected = 5 if mode == "timing" else 8
        if len(fields) != expected:
            raise RuntimeError("reference row shape changed")
        row = {"mode": mode, "workload": fields[1], "iterations": int(fields[2]), "repetition": int(fields[3])}
        numbers = [int(value) for value in fields[4:]]
        if mode == "timing":
            row["median_ns"] = numbers[0]
        else:
            row.update(zip(["allocations", "reallocations", "retained_bytes", "peak_extra_bytes"], numbers))
        rows.append(row)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--cargo-home", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--hostname-manifest", type=Path)
    parser.add_argument("--format-ab", action="store_true")
    parser.add_argument("--cached-node", type=Path)
    parser.add_argument("--node", default="node")
    parser.add_argument("--iterations", type=int, default=256)
    args = parser.parse_args()
    if args.iterations <= 0:
        parser.error("iterations must be positive")
    if args.format_ab and (args.cached_node is None or args.hostname_manifest is not None):
        parser.error("format A/B requires cached Node data and no hostname dependency")
    args.target_dir = args.target_dir.resolve(strict=True)
    args.cargo_home = args.cargo_home.resolve(strict=True)
    args.output_dir = args.output_dir.resolve()
    args.output_dir.relative_to(args.target_dir)
    args.output_dir.mkdir(parents=True, exist_ok=False)
    if args.hostname_manifest is not None:
        args.hostname_manifest = args.hostname_manifest.resolve(strict=True)
    node = None if args.format_ab else shutil.which(args.node)
    if not args.format_ab and node is None:
        raise RuntimeError("pinned Node executable is unavailable")
    env = compiler_environment(args)
    public = fixtures()
    rust_details = run(RUST + ["--version", "--verbose"], env)
    rust_version = next((line.split(": ", 1)[1] for line in rust_details.splitlines() if line.startswith("release: ")), None)
    rust_commit = next((line.split(": ", 1)[1] for line in rust_details.splitlines() if line.startswith("commit-hash: ")), None)
    if rust_version != "1.99.1" or rust_commit is None:
        raise RuntimeError("pinned Rust compiler identity differs")
    cached_node_sha256 = None
    if args.format_ab:
        cached = json.loads(args.cached_node.read_text())
        if (cached["warmups"], cached["repetitions"], cached["samples_per_repetition"], cached["node_version"]) != (8, 3, 31, "v24.11.1"):
            raise RuntimeError("cached Node protocol or verified version differs")
        for path in [HERE / "oracle.mjs", HERE / "corpus.tsv"]:
            if cached["inputs"][str(path.relative_to(ROOT))] != digest(path):
                raise RuntimeError("cached Node source or corpus differs")
        references = [row for row in cached["node"] if row["workload"] == "node-format-parts"]
        if len(references) != 3 or {row["repetition"] for row in references} != {0, 1, 2} or any(row["iterations"] != args.iterations for row in references):
            raise RuntimeError("cached Node iterations or samples differ")
        node_version = cached["node_version"]
        cached_node_sha256 = digest(args.cached_node)
    else:
        node_version = run([node, "--version"], public).strip()
        if node_version != "v24.11.1":
            raise RuntimeError("pinned Node version differs")
        run([node, str(HERE / "oracle.mjs")], public)
    compiled = []
    binaries = []
    # Direct std-only controls neither resolve nor link the hostname package.
    for name, flags in [("std-timing", []), ("std-allocations", ["--cfg", "allocation_admission"])]:
        binary, record = compile_reference(args, env, name, flags)
        compiled.append(record)
        binaries.append((binary, "allocations" if "allocations" in name else "timing"))
    extra, dependency = hostname_dependency(args, env)
    if extra:
        for name, flags in [("hostname-timing", extra), ("hostname-allocations", extra + ["--cfg", "allocation_admission"])]:
            binary, record = compile_reference(args, env, name, flags)
            compiled.append(record)
            binaries.append((binary, "allocations" if "allocations" in name else "timing"))
    results = []
    for binary, mode in binaries:
        run([str(binary)], public)
        operation = "--profile-format" if args.format_ab else "--profile-hostname" if binary.name.startswith("hostname-") else "--profile"
        quiet_guard()
        results.extend(rust_rows(run([str(binary), operation, str(args.iterations)], public), mode))
    node_results = []
    if not args.format_ab:
        quiet_guard()
        node_output = run([node, str(HERE / "oracle.mjs"), "--profile", str(args.iterations)], public)
        for line in node_output.splitlines():
            if line.startswith("B03 independent Node oracle checked "):
                continue
            fields = line.split("\t")
            if len(fields) != 4:
                raise RuntimeError("unexpected Node profile row")
            node_results.append({"workload": fields[0], "iterations": int(fields[1]), "repetition": int(fields[2]), "median_ns": int(fields[3])})
    inputs = [HERE / "reference.rs", HERE / "corpus.tsv", HERE / "oracle.mjs", HERE / "profile.py", ROOT / "crates/tsrust-compiler/src/stdlib/path/runtime.rs", ROOT / "crates/tsrust-compiler/src/stdlib/path/relative.rs"]
    print(json.dumps({"status": "admission-prototype", "rust_version": rust_version, "rust_commit": rust_commit, "node_version": node_version, "cached_node_sha256": cached_node_sha256, "warmups": 8, "repetitions": 3, "samples_per_repetition": 31, "inputs": {str(path.relative_to(ROOT)): digest(path) for path in inputs}, "compile": compiled, "hostname_dependency": dependency, "rust": results, "node": node_results}, indent=2))


if __name__ == "__main__":
    main()
