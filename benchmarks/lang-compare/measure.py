"""Run both builds of the CPU comparison and report per-phase medians.

Usage: python3 measure.py RUST_BINARY TSRUST_BINARY [RUNS]

Each run prints one line per phase: name, milliseconds, checksum. Engines
alternate between runs so a slow period on the machine affects both. The
script fails if any checksum differs between the two versions, then prints the
median time per phase, the Tsrust/Rust ratio, and median whole-process wall
time, CPU time and peak resident memory.
"""
import os
import re
import statistics
import subprocess
import sys


def run(binary):
    result = subprocess.run(["/usr/bin/time", "-l", binary], capture_output=True, text=True, env=dict(os.environ))
    if result.returncode != 0:
        raise SystemExit(f"{binary} exited with {result.returncode}:\n{result.stderr[-2000:]}")
    phases = {}
    for line in result.stdout.splitlines():
        name, milliseconds, checksum = line.split(" ")
        phases[name] = (float(milliseconds), checksum)
    real, user, system = (float(value) for value in re.search(r"([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys", result.stderr).groups())
    rss = int(re.search(r"(\d+)\s+maximum resident set size", result.stderr).group(1)) / (1024 * 1024)
    return phases, (real, user + system, rss)


def main():
    binaries = {"rust": sys.argv[1], "tsrust": sys.argv[2]}
    runs = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    samples = {name: [] for name in binaries}
    for _ in range(runs):
        for name, binary in binaries.items():
            samples[name].append(run(binary))
    reference = samples["rust"][0][0]
    for name, values in samples.items():
        for phases, _ in values:
            for phase, (_, checksum) in phases.items():
                if checksum != reference[phase][1]:
                    raise SystemExit(f"{name} {phase} checksum {checksum} differs from {reference[phase][1]}")
    print(f"{'phase':8} {'rust ms':>9} {'tsrust ms':>10} {'ratio':>6}   (median of {runs}, checksums equal)")
    for phase in reference:
        rust, tsrust = (statistics.median(phases[phase][0] for phases, _ in samples[name]) for name in binaries)
        print(f"{phase:8} {rust:9.1f} {tsrust:10.1f} {tsrust / rust:6.2f}")
    for name, values in samples.items():
        wall, cpu, rss = (statistics.median(column) for column in zip(*(process for _, process in values)))
        print(f"{name:7} process wall {wall:6.2f} s  cpu {cpu:6.2f} s  peak rss {rss:7.1f} MiB  binary {os.path.getsize(binaries[name]) / (1024 * 1024):.1f} MiB")


if __name__ == "__main__":
    main()
