"""Run both builds of the comparison app and report runtime measurements.

Usage: python3 measure.py RUST_BINARY TSRUST_BINARY [RUNS]

Each run sets BENCH_MODE, so the app quits by itself:
- startup: quits on the first 16 ms tick after the window opens. The harness
  activates each app as soon as it registers, so startup includes that call.
- idle: quits after 10 seconds with the window open and nothing changing.
- animate: refilters the 2,000-row list every 16 ms for 600 ticks.
Reported per mode: median wall seconds, median CPU seconds (user + system) and
median peak resident memory. Engines alternate between runs so a slow period on
the machine affects both.
"""
import os
import re
import statistics
import subprocess
import sys
import time


def run(binary, mode):
    env = dict(os.environ, BENCH_MODE=mode)
    process = subprocess.Popen(["/usr/bin/time", "-l", binary], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    # Neither app activates itself; bring each to the front the same way so
    # its window is visible and renders.
    for _ in range(100):
        child = subprocess.run(["pgrep", "-P", str(process.pid)], capture_output=True, text=True).stdout.split()
        if child and subprocess.run(["osascript", "-e", f'tell application "System Events" to set frontmost of (first process whose unix id is {child[0]}) to true'], capture_output=True).returncode == 0:
            break
        time.sleep(0.05)
    try:
        _, text = process.communicate(timeout=120)
    except subprocess.TimeoutExpired:
        process.kill()
        raise SystemExit(f"{binary} {mode} did not quit")
    if process.returncode != 0:
        raise SystemExit(f"{binary} {mode} exited with {process.returncode}:\n{text[-2000:]}")
    real, user, system = (float(value) for value in re.search(r"([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys", text).groups())
    rss = int(re.search(r"(\d+)\s+maximum resident set size", text).group(1))
    return real, user + system, rss / (1024 * 1024)


def main():
    rust, tsrust, runs = sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 5
    engines = {"rust": rust, "tsrust": tsrust}
    for name, binary in engines.items():
        print(f"{name}: binary {os.path.getsize(binary) / (1024 * 1024):.1f} MiB")
    for mode, count in (("startup", runs), ("idle", max(2, runs // 2)), ("animate", max(2, runs // 2))):
        samples = {name: [] for name in engines}
        for _ in range(count):
            for name, binary in engines.items():
                samples[name].append(run(binary, mode))
        for name, values in samples.items():
            wall, cpu, rss = (statistics.median(column) for column in zip(*values))
            print(f"{mode:8} {name:7} wall {wall:6.2f} s  cpu {cpu:6.2f} s  peak rss {rss:7.1f} MiB  (n={len(values)})", flush=True)


if __name__ == "__main__":
    main()
