#!/usr/bin/env python3
"""Serial native-family acceptance, with bounded owned process trees. Not a benchmark."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import tomllib


COMPILER_TARGETS = """std_bytes byte_operations byte_codecs byte_codec_export
byte_operation_export native_path native_path_relative path_env_os environment service_sections
async_io async_io_runtime task_runtime task_ergonomics native_channels
native_threads native_threads_backend process process_runtime network_addresses
network_address_runtime network_transport network_transport_runtime compression
compression_runtime compression_streams_runtime http""".split()
LSP_TARGETS = """byte_operations byte_codecs native_file_catalog
native_environment_catalog async_io async_tasks native_threads process
network_addresses network_transport compression stdio""".split()


def owned_rows(rows, pid):
    """Include group members and descendants, even if a descendant changed group."""
    owned = {pid}
    while True:
        added = {r["pid"] for r in rows if r["ppid"] in owned or r.get("pgid") == pid}
        if added <= owned:
            return [r for r in rows if r["pid"] in owned]
        owned |= added


def process_rows():
    if os.name == "nt":
        script = """$ErrorActionPreference='Stop'; $rows=@(Get-CimInstance Win32_Process | ForEach-Object {
          $entry=$_; $p=Get-Process -Id $entry.ProcessId -ErrorAction SilentlyContinue;
          if ($null -ne $p) { [pscustomobject]@{pid=[int]$entry.ProcessId; ppid=[int]$entry.ParentProcessId;
            born=$(if ($null -ne $entry.CreationDate) {$entry.CreationDate.ToUniversalTime().ToString('o')} else {''});
            name=[string]$entry.Name;
            bytes=[Math]::Max($p.WorkingSet64,$p.PrivateMemorySize64)} }
        }); ConvertTo-Json -InputObject $rows -Compress"""
        output = subprocess.check_output(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script], text=True, timeout=15)
        return json.loads(output)
    output = subprocess.check_output(["ps", "-A", "-o", "pid=,ppid=,pgid=,rss=,lstart=,stat="], text=True, timeout=10)
    rows = []
    libproc = ctypes.CDLL("/usr/lib/libproc.dylib") if sys.platform == "darwin" else None
    for line in output.splitlines():
        pid, ppid, pgid, rss, *tail = line.split()
        born, status = " ".join(tail[:-1]), tail[-1]
        if status.startswith("Z"):
            continue
        size = int(rss) * 1024
        if libproc:
            usage = (ctypes.c_uint64 * 12)()
            if libproc.proc_pid_rusage(int(pid), 0, ctypes.byref(usage)) == 0:
                size = max(size, usage[9])
        rows.append({"pid": int(pid), "ppid": int(ppid), "pgid": int(pgid), "born": born, "bytes": size})
    return rows


def kill_owned(process, seen):
    if os.name == "nt":
        # taskkill handles the still-live root tree; retained creation identities
        # also cover children whose immediate parent has already exited.
        if process.poll() is None:
            subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True, timeout=20)
        current = {r["pid"]: r for r in process_rows()} if seen else {}
        for pid, row in seen.items():
            if pid in current and current[pid]["born"] == row["born"]:
                subprocess.run(["taskkill", "/PID", str(pid), "/T", "/F"], capture_output=True, timeout=20)
        if process.poll() is None:
            process.kill()  # Popen's Windows handle cannot target a reused PID.
    else:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        current = {r["pid"]: r for r in process_rows()} if seen else {}
        for pid, row in seen.items():
            if pid in current and current[pid]["born"] == row["born"]:
                try:
                    os.kill(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
    process.wait(timeout=20)


def bounded(command, cwd, env, prefix, limit_seconds=1800, limit_bytes=5_000_000 * 1024):
    """A failed counter is a stop condition, never unbounded execution."""
    start = time.monotonic()
    peak = 0
    reason = None
    seen = {}
    with prefix.with_suffix(".out").open("w") as out, prefix.with_suffix(".err").open("w") as err:
        options = {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == "nt" else {"start_new_session": True}
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=out, stderr=err, **options)
        try:
            while process.poll() is None:
                rows = process_rows()
                selected = owned_rows(rows, process.pid)
                selected += [r for r in rows if r["pid"] in seen and r["born"] == seen[r["pid"]]["born"] and r not in selected]
                seen.update({r["pid"]: r for r in selected})
                peak = max(peak, sum(r["bytes"] for r in selected))
                if peak > limit_bytes or time.monotonic() - start > limit_seconds:
                    reason = "memory" if peak > limit_bytes else "timeout"
                    break
                time.sleep(1 if os.name == "nt" else 0.5)
            rows = process_rows()
            remaining = owned_rows(rows, process.pid)
            remaining += [r for r in rows if r["pid"] in seen and r["born"] == seen[r["pid"]]["born"] and r not in remaining]
            remaining = [r for r in remaining if r["pid"] != process.pid]
            if remaining and reason is None:
                reason = "unsettled child processes"
            if reason:
                seen.update({r["pid"]: r for r in remaining})
                kill_owned(process, seen)
        except BaseException:
            kill_owned(process, seen)
            raise
    return {"code": process.returncode, "stop": reason, "seconds": round(time.monotonic() - start, 3),
            "rootPid": process.pid, "root": seen.get(process.pid), "remaining": remaining,
            "peakBytes": peak, "stdout": str(prefix.with_suffix(".out")), "stderr": str(prefix.with_suffix(".err"))}


def sha(path):
    with Path(path).open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", required=True, choices=["x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-pc-windows-msvc"])
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = root / "target"
    version = tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    cargo = ["rustup", "run", version, "cargo"]
    suffix = ".exe" if os.name == "nt" else ""
    env = {**os.environ, "CARGO_TARGET_DIR": str(target), "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "1",
           "TSRUST_LSP_SERVER": str(target / "debug" / ("tsrust-lsp" + suffix))}
    for key in ["TSRUST_LSP_MATRIX_ROW", "TSRUST_LSP_AGREEMENT_TYPE", "TSRUST_LSP_MATRIX_REPORT", "TSRUST_SIGNAL_DELIVERY_HELPER_PID", "TSRUST_SIGNAL_DELIVERY_HELPER_EVENT"]:
        env.pop(key, None)
    host = subprocess.check_output(["rustup", "run", version, "rustc", "-vV"], cwd=root, env=env, text=True)
    assert "host: " + args.host in host.splitlines(), host
    assert subprocess.check_output(["node", "--version"], text=True).strip() == "v24.11.1"
    checkpoint = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    def frozen():
        assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip() == checkpoint
        assert not subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=root, text=True).strip(), "Commit source before platform acceptance"
    frozen()
    destination = target / "platform-acceptance" / (args.host + "-" + checkpoint[:12])
    destination.mkdir(parents=True, exist_ok=True)
    result_path = destination / "results.json"
    results = json.loads(result_path.read_text()) if result_path.exists() else {}
    assert not results or results["checkpoint"] == checkpoint
    results.update(checkpoint=checkpoint, host=args.host, complete=False)
    def save():
        temporary = result_path.with_suffix(".tmp")
        temporary.write_text(json.dumps(results, indent=2) + "\n")
        temporary.replace(result_path)
    save()
    def run(label, command, cwd=root):
        frozen()
        # Unique attempts preserve failed-scope evidence without overwriting logs.
        prefix = destination / (label + "-" + str(time.time_ns()))
        try:
            row = bounded(command, cwd, env, prefix)
            frozen()
        except BaseException as error:
            results.setdefault("attempts", []).append({"label": label, "stop": type(error).__name__, "message": str(error)})
            save()
            raise
        results.setdefault("attempts", []).append({"label": label, **row})
        save()
        if row["stop"]:
            raise RuntimeError(f"{label}: watchdog stopped: {row['stop']}")
        return row
    row = run("consumers", cargo + ["build", "--offline", "--locked", "-p", "tsrust-compiler", "-p", "tsrust-lsp", "--bin", "tsrust", "--bin", "tsrust-lsp"])
    assert row["code"] == 0, row
    def consumers():
        return {name: sha(target / "debug" / (name + suffix)) for name in ["tsrust", "tsrust-lsp"]}
    row = run("portable", [str(target / "debug" / ("tsrust" + suffix)), "setup", "--portable", "--offline", "--yes"])
    assert row["code"] == 0, row
    failures = []
    for package, targets in [("tsrust-compiler", COMPILER_TARGETS), ("tsrust-lsp", LSP_TARGETS)]:
        for name in targets:
            key = package + "::" + name
            row = run(package + "-" + name + "-build", cargo + ["test", "--offline", "--locked", "-p", package, "--test", name, "--no-run", "--message-format=json"])
            messages = [json.loads(line) for line in Path(row["stdout"]).read_text().splitlines() if line.startswith("{")]
            if row["code"] or not messages or messages[-1].get("reason") != "build-finished" or messages[-1].get("success") is not True:
                failures.append(key + "::build")
                continue
            artifacts = [m for m in messages if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["name"] == name and "test" in m["target"]["kind"] and m["profile"]["test"]]
            assert len(artifacts) == 1, artifacts
            binary = Path(artifacts[0]["executable"])
            identity = {"checkpoint": checkpoint, "binary": sha(binary), "consumers": consumers()}
            previous = results.setdefault("targets", {}).get(key, {})
            tests = previous.get("tests", {}) if previous.get("identity") == identity else {}
            results["targets"][key] = {"identity": identity, "tests": tests}
            package_root = root / "crates" / package
            row = run(package + "-" + name + "-list", [str(binary), "--list", "--format", "terse"], package_root)
            assert row["code"] == 0, row
            names = [line[:-6] for line in Path(row["stdout"]).read_text().splitlines() if line.endswith(": test")]
            assert names, key
            for test in names:
                if name == "service_sections" and test == "delivered_signals::internal_owned_console_signal_sender":
                    tests[test] = {"status": "excluded", "reason": "Internal ignored console sender runs only as the generated listener test's owned helper; not an independent delivery witness."}
                    save()
                    continue
                if package == "tsrust-compiler" and name == "http" and test == "sql_and_https_features_share_one_generated_program_without_provider_ambiguity":
                    tests[test] = {"status": "excluded", "reason": "Opt-in SQL/Docker acceptance is separate from this native I/O family."}
                    save()
                    continue
                large_datagram = name == "network_transport_runtime" and test == "udp_large_datagram_delivers_all_60000_bytes"
                if large_datagram and args.host != "x86_64-unknown-linux-gnu":
                    tests[test] = {"status": "excluded", "reason": "The supporting-host 60000-byte datagram witness runs on Linux; this is not a delivery pass."}
                    save()
                    continue
                if tests.get(test, {}).get("success"):
                    continue
                assert sha(binary) == identity["binary"]
                assert consumers() == identity["consumers"]
                command = [str(binary), "--exact", test, "--test-threads=3"] + (["--ignored"] if large_datagram else [])
                row = run(package + "-" + name + "-" + hashlib.sha256(test.encode()).hexdigest()[:12], command, package_root)
                assert sha(binary) == identity["binary"] and consumers() == identity["consumers"]
                text = Path(row["stdout"]).read_text(errors="replace")
                row["success"] = row["code"] == 0 and "test result: ok. 1 passed;" in text
                tests[test] = row
                save()
                print(key, test, "passed" if row["success"] else "FAILED", flush=True)
                if not row["success"]:
                    failures.append(key + "::" + test)
            if all(tests.get(test, {}).get("success") or tests.get(test, {}).get("status") == "excluded" for test in names):
                binary.unlink()
    results["failures"] = failures
    save()
    assert not failures, failures
    with tempfile.TemporaryDirectory(prefix="package-", dir=destination) as temporary:
        package = Path(temporary) / "toolchain"
        row = run("package", ["node", "tools/package-toolchain.mjs", str(target / "debug" / ("tsrust" + suffix)), str(target / "debug" / ("tsrust-lsp" + suffix)), args.host, str(package)])
        assert row["code"] == 0, row
        row = run("packaged-smoke", ["node", "tools/smoke-toolchain.mjs", str(package / "bin" / ("tsrust" + suffix)), str(package / "bin" / ("tsrust-lsp" + suffix))])
        assert row["code"] == 0, row
    frozen()
    results["complete"] = True
    save()


if __name__ == "__main__":
    def interrupted(number, frame):
        raise SystemExit(128 + number)
    signal.signal(signal.SIGTERM, interrupted)
    main()
