#!/usr/bin/env python3
"""Assign every compiler/LSP test target to one of COUNT parallel full-regression shards.

Each shard runs on its own runner with its own target directory and one-hour job
ceiling. Shards run in parallel, never serially. Weights only balance them, so a
stale or missing weight changes balance and never omits a target.

usage: regression-shards.py SHARD COUNT   (prints one `cargo test` argument line per package)
"""
import json
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ["tsrust-compiler", "tsrust-lsp"]


def targets():
    return [(package, path.stem) for package in PACKAGES for path in sorted((ROOT / "crates" / package / "tests").glob("*.rs"))]


def shards(count):
    weights = json.loads((ROOT / "tools/regression-weights.json").read_text())["seconds"]
    default = statistics.median(weights.values())
    # Library, binary and doctest units always run on shard 0.
    loads, plan = [weights.get("units", 900)] + [0] * (count - 1), [[] for _ in range(count)]
    for package, name in sorted(targets(), key=lambda item: (-weights.get("::".join(item), default), item)):
        index = loads.index(min(loads))
        loads[index] += weights.get(package + "::" + name, default)
        plan[index].append((package, name))
    return plan, loads


def main():
    shard, count = int(sys.argv[1]), int(sys.argv[2])
    assert 0 <= shard < count
    selected = shards(count)[0][shard]
    for package in PACKAGES:
        names = sorted(name for owner, name in selected if owner == package)
        units = ["--lib", "--bins"] if shard == 0 else []
        if names or units:
            print(" ".join(["-p", package, *units, *(f"--test {name}" for name in names)]))


if __name__ == "__main__":
    main()
