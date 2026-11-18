"""Measure the edit loop of both versions of the comparison app.

Usage: python3 edit_loop.py TSRUST_EXECUTABLE [EDITS]

Each edit changes the row's muted label ("chars") to a new, never-seen string,
so no cache of an earlier revision applies; this is the rebuild after a real
edit. Rust: `cargo build` (dev profile, incremental) after editing src/row.rs,
then `cargo check`. Tsrust: one `tsrust dev --verbose` session; each edit of
src/row.tsrx is timed by its "Rebuilt in" line (dev launches the program with
BENCH_MODE=startup, so it quits at once), then `tsrust check`.
Edits are restored afterwards.
"""
import os
import re
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.join(HERE, "rust")
TSRUST = os.path.join(HERE, "tsrust")


def timed(command, cwd):
    started = time.perf_counter()
    subprocess.run(command, cwd=cwd, check=True, capture_output=True, env=dict(os.environ, CARGO_TERM_COLOR="never"))
    return time.perf_counter() - started


def replace(path, old, new):
    with open(path) as file:
        text = file.read()
    assert old in text, (path, old)
    with open(path, "w") as file:
        file.write(text.replace(old, new, 1))


def rust_loop(edits):
    row = os.path.join(RUST, "src", "row.rs")
    builds, checks, previous = [], [], "chars"
    for index in range(edits):
        label = f"chars{index}"
        replace(row, f'"{{}} {previous}"', f'"{{}} {label}"')
        previous = label
        builds.append(timed(["cargo", "build", "--offline"], RUST))
        replace(row, f'"{{}} {previous}"', f'"{{}} {previous}x"')
        previous += "x"
        checks.append(timed(["cargo", "check", "--offline"], RUST))
    replace(row, f'"{{}} {previous}"', '"{} chars"')
    timed(["cargo", "build", "--offline"], RUST)
    return builds, checks


def tsrust_loop(executable, edits):
    row = os.path.join(TSRUST, "src", "row.tsrx")
    log_path = os.path.join(TSRUST, "dev.log")
    log = open(log_path, "w")
    dev = subprocess.Popen([executable, "dev", "--verbose"], cwd=TSRUST, stdout=log, stderr=subprocess.STDOUT, env=dict(os.environ, BENCH_MODE="startup"))

    def built(count):
        deadline = time.time() + 900
        while time.time() < deadline:
            with open(log_path) as file:
                text = file.read()
            if "Build failed" in text:
                raise SystemExit("tsrust dev build failed:\n" + text[-3000:])
            found = re.findall(r"(?:Built|Rebuilt) in (\d+) ms", text)
            if len(found) >= count:
                return int(found[count - 1])
            time.sleep(0.2)
        raise SystemExit("tsrust dev did not finish a build:\n" + text[-3000:])

    try:
        first = built(1)
        rebuilds, checks, previous = [], [], "chars"
        for index in range(edits):
            label = f"chars{index}"
            replace(row, f"' {previous}'", f"' {label}'")
            previous = label
            time.sleep(0.5)
            rebuilds.append(built(index + 2) / 1000)
        replace(row, f"' {previous}'", "' chars'")
        built(edits + 2)
    finally:
        dev.terminate()
        dev.wait()
        log.close()
    for index in range(edits):
        replace(row, "' chars'", f"' chars{index}y'")
        checks.append(timed([executable, "check", "src/main.tsrx"], TSRUST))
        replace(row, f"' chars{index}y'", "' chars'")
    return first / 1000, rebuilds, checks


def summary(values):
    return f"median {statistics.median(values):6.2f} s  min {min(values):6.2f}  max {max(values):6.2f}  (n={len(values)})"


def main():
    executable, edits = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 5
    builds, checks = rust_loop(edits)
    print(f"rust   cargo build after edit  {summary(builds)}", flush=True)
    print(f"rust   cargo check after edit  {summary(checks)}", flush=True)
    first, rebuilds, checks = tsrust_loop(executable, edits)
    print(f"tsrust dev first build         {first:6.2f} s", flush=True)
    print(f"tsrust dev rebuild after edit  {summary(rebuilds)}", flush=True)
    print(f"tsrust check after edit        {summary(checks)}", flush=True)


if __name__ == "__main__":
    main()
