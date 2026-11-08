#!/usr/bin/env python3
"""Serial native-family acceptance, with bounded owned process trees. Not a benchmark."""
import argparse
import ctypes
from datetime import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import time
import tomllib


COMPILER_TARGETS = """std_fs native_file_cleanup native_full_files native_file_writes
native_filesystem_utilities native_buffered_files native_buffered_file_backend
std_bytes byte_operations byte_codecs byte_codec_export
byte_operation_export native_path native_path_relative path_env_os environment service_sections
async_io async_io_runtime task_runtime task_ergonomics native_channels
native_threads native_threads_backend process process_runtime network_addresses
network_address_runtime network_transport network_transport_runtime compression
compression_runtime compression_streams_runtime http
url_values url_features regex_text explicit_locale crypto_primitives structured_logging realtime owned_iterators stdlib_completion rust_adapter_generation rust_bridge""".split()
LSP_TARGETS = """byte_operations byte_codecs native_file_catalog
native_environment_catalog async_io async_tasks native_threads process
network_addresses network_transport compression stdio
url crypto structured_logging rust_bridge""".split()


# One reviewed repair of run 42460717406; prior passes keep their old identity.
REPAIR_CHECKPOINT = "469e8bb32a86771eb3317cd3232554c6a4192752"
REPAIR_RESULTS = {
    "x86_64-unknown-linux-gnu": "fdc9d24caca9f9f6368cce792192932cf223f91d3afcad57089747aeb21ab5e3",
    "aarch64-apple-darwin": "ce95dc885f43006d06d622bfa538b69d6832f6f8bd30b943b2a2f8e261d18dba",
    "x86_64-apple-darwin": "854b7e55f8c8adf3d2d1e6e8bfc850aa88bf1e77707aff58db4dd44bafc23bd3",
    "x86_64-pc-windows-msvc": "3a6fb2f204ec44c00b02ac83b8e4b01dc30fcc39a4bb81325c01d00693d77c00",
}
# Reviewed second run; each host is sealed independently.
SECOND_CHECKPOINT = "f46d740023318f239e04ce190b1461c08f7272b5"
SECOND_RESULTS = {
    "x86_64-apple-darwin": "0ca76db650f65eb7d0728a2aa3721ee6c0f2b1213da76a3f603a9d69c5ecd6fd",
    "x86_64-unknown-linux-gnu": "a90b6deb080bc6da0bc02f57f4c42fc60b67e699c8f4bbae1780867c2f08a4db",
    "x86_64-pc-windows-msvc": "a84b5eb1c71682e10a2ef5e4da8136e7b77fd301b3a28d5801ad44afec61b28e",
    "aarch64-apple-darwin": "1155d283594b0ce715890565209ca351e7c5cfece6993064a3e5f021d700eea6",
}


def second_repair_plan(path, host):
    path = Path(path)
    if path.is_dir():
        paths = list(path.rglob("results.json"))
        assert len(paths) == 1, "Expected one second-run host result"
        path = paths[0]
    assert host in SECOND_RESULTS, "Second-run host evidence not yet reviewed"
    assert sha(path) == SECOND_RESULTS[host], "Second artifact results changed"
    prior = json.loads(path.read_text())
    assert prior["host"] == host and prior["checkpoint"] == SECOND_CHECKPOINT
    assert prior["retainedEvidence"] == {"checkpoint": REPAIR_CHECKPOINT, "resultsSha256": REPAIR_RESULTS[host], "run": "42460717406"}
    return prior


def second_retained_cases(prior, key, names):
    return own_passes(prior, key, names, SECOND_CHECKPOINT)

# Reviewed third run at a7795eea; its own passes keep that identity.
THIRD_CHECKPOINT = "a7795eea73d631a9f4340c0c5d898a107ac047db"
THIRD_RESULTS = {
    "x86_64-unknown-linux-gnu": "3c28afa0fa412413345f798a29737d3a41e59ff3894d17e435cde4969da2ca05",
    "aarch64-apple-darwin": "849cf27d2e778b4fa51e2c0999ab80be59d6971b20afb04bdf832f56c2be950e",
    "x86_64-apple-darwin": "8ef7dc2d85f5b32bd9ee7cbf304005e18f4f05a764fd249f5c1c804aed0715d3",
    "x86_64-pc-windows-msvc": "5f3be8021b04ff80045fb2e4487de9b509a96034d7bba47c6c7b8b522a8759ed",
}


def third_repair_plan(path, host):
    path = Path(path)
    if path.is_dir():
        paths = list(path.rglob("results.json"))
        assert len(paths) == 1, "Expected one third-run host result"
        path = paths[0]
    assert host in THIRD_RESULTS, "Third-run host evidence not yet reviewed"
    assert sha(path) == THIRD_RESULTS[host], "Third artifact results changed"
    prior = json.loads(path.read_text())
    assert prior["host"] == host and prior["checkpoint"] == THIRD_CHECKPOINT
    assert prior["retainedEvidence"] == {"checkpoint": REPAIR_CHECKPOINT, "resultsSha256": REPAIR_RESULTS[host], "run": "42460717406"}
    assert prior["secondRetainedEvidence"] == {"checkpoint": SECOND_CHECKPOINT, "resultsSha256": SECOND_RESULTS[host], "run": "42470181942"}
    return prior


# Reviewed later runs in order; each must carry every earlier seal unchanged.
LATER_RUNS = [
    {"run": "42480586761", "checkpoint": "3d6767c53984f81eddca9b8770d81b64f654975c", "results": {
        "x86_64-unknown-linux-gnu": "720c3bd41c0d1fc70e30d160d93acfe18f0795cd0eca56122a8489c3a1704e9d",
        "aarch64-apple-darwin": "377d65d1f09567c6c672ddd6992d0b9f97346d5518277f6db33be2ed29948d9a",
        "x86_64-apple-darwin": "13a54555998dde8b60f5110343872b9cfc21c34b72f9b876c98e1668094c3d8a",
        "x86_64-pc-windows-msvc": "e14c5b60e82c26af3b7076706a8120c457ad02282f57d6bd160d83dce9b55b58",
    }},
    {"run": "42485215015", "checkpoint": "a341f7ffed285d1adcde661792c659b03e123028", "results": {
        "x86_64-unknown-linux-gnu": "33f3a296d0b9237ef48b8eab3c96b6045d4dde30beffb4f1564633d2fa7e95b9",
        "aarch64-apple-darwin": "1be2637ef4645a2b20f82786dd94092a60c4f5aee06b52083091eb6e370b26aa",
        "x86_64-apple-darwin": "86cca64f61d87778be1c17ad0850be3ed2ea6e7ae3be7b5fbc066dfb9872f349",
        "x86_64-pc-windows-msvc": "ef36ee034f8c223a08cb79d6c9a228e90f726fc6123cc0ca4940ab805c7e0084",
    }},
    {"run": "42494556962", "checkpoint": "3972fa49524bf0521d6d9484fb9fee46a77ef816", "results": {
        "x86_64-unknown-linux-gnu": "9e39f0fc66c482bb62320d568574662fb3d3d4e9cabdd8addf2e25090a89cbd9",
        "aarch64-apple-darwin": "5a4fc8a2ce1e422d247e2a6ef598abb0b8bc6171759d5d6e68879e5f2e3a63c7",
        "x86_64-apple-darwin": "98908cee0d552db38ad534a55f5e54bb4084fde3d6cc5925101f46d04def97ca",
        "x86_64-pc-windows-msvc": "af07e6fb4573d7fb33876f130f7aff761e49f900eeadf91ab1918a8852880b21",
    }},
    {"run": "42501171400", "checkpoint": "895e8c915a9e169467f006ebd4c1b5b7854d2daa", "results": {
        "x86_64-unknown-linux-gnu": "d2961cde8e970d2b1dd78a11d274ba8f123baa7413fc3822c56b7b3a461ce3f0",
        "aarch64-apple-darwin": "6650c6bfc51e17059b6c5b8eab2b3acc411959c266e1fb1670775cd982944aa0",
        "x86_64-apple-darwin": "7038ac70287ad0e2ad7fabca07a780bc808d035916ae1ccbedd0605fcf48a1d3",
        "x86_64-pc-windows-msvc": "383b73ad9db4d501cb3ae577d337ff80a1ceaaa88d419a5f945bab22a238476d",
    }},
]


def earlier_seals(host, count):
    return {"retainedEvidence": {"checkpoint": REPAIR_CHECKPOINT, "resultsSha256": REPAIR_RESULTS[host], "run": "42460717406"},
            "secondRetainedEvidence": {"checkpoint": SECOND_CHECKPOINT, "resultsSha256": SECOND_RESULTS[host], "run": "42470181942"},
            "thirdRetainedEvidence": {"checkpoint": THIRD_CHECKPOINT, "resultsSha256": THIRD_RESULTS[host], "run": "42474075588"},
            "laterRetainedEvidence": [{"checkpoint": later["checkpoint"], "resultsSha256": later["results"][host], "run": later["run"]} for later in LATER_RUNS[:count]]}


def later_repair_plan(path, host, index):
    later = LATER_RUNS[index]
    paths = list(Path(path).rglob("results.json")) if Path(path).is_dir() else [Path(path)]
    assert len(paths) == 1, "Expected one later-run host result"
    assert host in later["results"], "Later-run host evidence not yet reviewed"
    assert sha(paths[0]) == later["results"][host], "Later artifact results changed"
    prior = json.loads(paths[0].read_text())
    assert prior["host"] == host and prior["checkpoint"] == later["checkpoint"]
    for key, value in earlier_seals(host, index).items():
        assert prior.get(key, []) == value, f"Later run {later['run']} changed {key}"
    return prior


def own_passes(prior, key, names, checkpoint):
    previous = prior.get("targets", {}).get(key, {})
    if not previous:
        return set()
    assert previous["identity"]["checkpoint"] == checkpoint
    # Never read nested retained cases here: those belong to earlier runs.
    return {name for name in names if (row := previous.get("tests", {}).get(name, {})).get("success") is True
            and row.get("status") == "ok" and row.get("scopeStop") is None}

REPAIR_REPEAT = {
    "tsrust-compiler::structured_logging": {"public_registered_suites_share_one_native_build", "relocated_logging_suite_executes_offline_at_both_optimizations"},
    "tsrust-compiler::realtime": {"relocated_realtime_suite_executes_offline_at_both_optimizations"},
    "tsrust-compiler::rust_adapter_generation": {"generated_nominal_record_projections_move_nested_string_buffers_without_clone"},
    "tsrust-compiler::rust_bridge": {"rust_owned_buffers_results_rayon_cancellation_and_cleanup"},
    "tsrust-compiler::async_io": {"registered_async_io_runtime_family_executes_at_both_required_profiles", "stored_io_calls_preserve_laziness_optional_arguments_views_and_two_loans"},
    "tsrust-compiler::task_ergonomics": {"registered_task_runtime_examples_share_one_profiled_runner"},
    "tsrust-lsp::async_io": {"io_explicit_result_signature_and_docs_agree_in_all_server_phases"},
}


def repair_plan(path, host):
    path = Path(path)
    if path.is_dir():
        paths = list(path.rglob("results.json"))
        assert len(paths) == 1, "Expected one prior host result"
        path = paths[0]
    assert sha(path) == REPAIR_RESULTS[host], "Prior artifact results changed"
    prior = json.loads(path.read_text())
    assert prior["host"] == host and prior["checkpoint"] == REPAIR_CHECKPOINT
    return prior


def retained_cases(prior, key, names):
    assert REPAIR_REPEAT.get(key, set()) <= set(names), "Required repair witness disappeared"
    previous = prior.get("targets", {}).get(key, {})
    if not previous:
        return set()  # Every case of an unreached target must execute.
    assert previous["identity"]["checkpoint"] == REPAIR_CHECKPOINT
    return {name for name in names if name not in REPAIR_REPEAT.get(key, set())
            and (row := previous.get("tests", {}).get(name, {})).get("success") is True
            and row.get("status") == "ok" and row.get("scopeStop") is None}


def selected_test_artifacts(messages, root):
    assert messages and messages[-1].get("reason") == "build-finished" and messages[-1].get("success") is True, "Selected test build did not finish successfully"
    expected = {(package, name) for package, names in [("tsrust-compiler", COMPILER_TARGETS), ("tsrust-lsp", LSP_TARGETS)] for name in names}
    selected = {(package, name) for package in ["tsrust-compiler", "tsrust-lsp"] for name in set(COMPILER_TARGETS + LSP_TARGETS) if (root / "crates" / package / "tests" / (name + ".rs")).is_file()}
    assert expected <= selected
    packages = {}
    for package in ["tsrust-compiler", "tsrust-lsp"]:
        directory = root / "crates" / package
        version = tomllib.loads((directory / "Cargo.toml").read_text())["package"]["version"]
        packages["path+" + directory.as_uri() + "#" + version] = package
    artifacts, normal = {}, {}
    for message in messages:
        if message.get("reason") != "compiler-artifact":
            continue
        if message.get("target", {}).get("kind") == ["bin"] and message["target"]["name"] in ["tsrust", "tsrust-lsp"]:
            name = message["target"]["name"]
            package, source = ("tsrust-compiler", "src/cli/main.rs") if name == "tsrust" else ("tsrust-lsp", "src/main.rs")
            assert packages.get(message["package_id"]) == package and name not in normal
            assert message["manifest_path"] == str(root / "crates" / package / "Cargo.toml")
            assert message["target"]["src_path"] == str(root / "crates" / package / source) and message["profile"]["test"] is False
            assert message["executable"] == str(root / "target/debug" / (name + (".exe" if os.name == "nt" else "")))
            normal[name] = message["executable"]
            continue
        if "test" not in message.get("target", {}).get("kind", []):
            continue
        package = packages.get(message["package_id"])
        name = message["target"]["name"]
        key = (package, name)
        assert key in selected and key not in artifacts, ("Unexpected or duplicate test artifact", key)
        assert message["manifest_path"] == str(root / "crates" / package / "Cargo.toml")
        assert message["target"]["src_path"] == str(root / "crates" / package / "tests" / (name + ".rs"))
        assert message["target"]["kind"] == ["test"] and message["target"]["test"] is True and message["profile"]["test"] is True
        assert message.get("executable"), key
        binary = Path(message["executable"])
        assert binary.parent == root / "target/debug/deps", binary
        artifacts[key] = binary
    assert set(normal) == {"tsrust", "tsrust-lsp"}, "Missing normal CLI/LSP artifacts"
    assert set(artifacts) == selected, ("Missing selected test artifacts", selected - set(artifacts))
    assert len(set(artifacts.values())) == len(selected), "Selected test executables must be distinct"
    return {key: artifacts[key] for key in expected}


def target_statuses(text, names, require_summary):
    assert names and len(set(names)) == len(names)
    statuses, problems = {}, []
    counts = re.findall(r"^running (\d+) tests?$", text.split("\nfailures:\n", 1)[0], re.M)
    if counts != [str(len(names))]:
        problems.append("Selected test count mismatch")
    status_output = text.split("\nfailures:\n", 1)[0]
    for name, status in re.findall(r"^test (.+?) \.\.\. (ok|FAILED|ignored(?:,.*)?)$", status_output, re.M):
        status = status.split(",", 1)[0]
        if name not in names and name.endswith(" - should panic") and name.removesuffix(" - should panic") in names:
            name = name.removesuffix(" - should panic")
        if name not in names:
            problems.append("Unexpected test: " + name)
        elif name in statuses:
            statuses[name] = "duplicate"
            problems.append("Duplicate test: " + name)
        else:
            statuses[name] = status
    for name in names:
        if statuses.get(name) != "ok":
            problems.append(name + ": " + statuses.get(name, "missing"))
    if require_summary:
        summaries = re.findall(r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text, re.M)
        observed = tuple(sum(value == status for value in statuses.values()) for status in ["ok", "FAILED", "ignored"])
        if not summaries or tuple(map(int, summaries[-1])) != observed or sum(observed) != len(names):
            problems.append("Test summary mismatch")
    return statuses, problems


def process_birth(row):
    # CIM supplies UTC ISO timestamps; ps lstart has host-local second precision.
    born = row["born"]
    if re.match(r"^\d{4}-\d{2}-\d{2}T", born):
        parsed = datetime.fromisoformat(born.replace("Z", "+00:00"))
        fraction = re.search(r"\.(\d+)", born)
        nanoseconds = int((fraction.group(1) if fraction else "").ljust(9, "0"))
        return int(parsed.replace(microsecond=0).timestamp()) * 1_000_000_000 + nanoseconds
    return int(time.mktime(time.strptime(born, "%a %b %d %H:%M:%S %Y"))) * 1_000_000_000


def owned_rows(rows, pid, created_after=None):
    """Require creation-ordered ancestry, including regrouped descendants."""
    root = next((row for row in rows if row["pid"] == pid), None)
    if created_after is None:
        if root is None:
            return []
        created_after = process_birth(root)
    owned = {pid: process_birth(root) if root is not None else created_after}
    while True:
        added = {row["pid"]: process_birth(row) for row in rows
                 if process_birth(row) >= created_after
                 and ((row["ppid"] in owned and process_birth(row) >= owned[row["ppid"]])
                      or row.get("pgid") == pid)}
        if added.keys() <= owned.keys():
            return [row for row in rows if row["pid"] in owned and process_birth(row) >= created_after]
        owned.update(added)


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
        # A Windows PPID can refer to a reused process ID. Never taskkill /T:
        # it can reach unrelated older processes even after our tree was filtered.
        if process.poll() is None:
            process.kill()  # Popen's owned Windows handle cannot target a reused PID.
        current = {r["pid"]: r for r in process_rows()} if seen else {}
        for pid, row in seen.items():
            if pid != process.pid and pid in current and current[pid]["born"] == row["born"]:
                subprocess.run(["taskkill", "/PID", str(pid), "/F"], capture_output=True, timeout=20)
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


def bounded(command, cwd, env, prefix, limit_seconds=1800, limit_bytes=5_000_000 * 1024, settle_build_children=False):
    """A failed counter is a stop condition, never unbounded execution."""
    start = time.monotonic()
    peak = 0
    reason = None
    seen = {}
    cleanup = None
    with prefix.with_suffix(".out").open("w") as out, prefix.with_suffix(".err").open("w") as err:
        # A new Windows process group would make every descendant inherit
        # "ignore Ctrl+C", hiding real console-signal delivery. Windows cleanup
        # uses the owned handle and exact PID/birth rows, not a group.
        options = {} if os.name == "nt" else {"start_new_session": True}
        created_after = time.time_ns() // 100 * 100 if os.name == "nt" else int(time.time()) * 1_000_000_000
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=out, stderr=err, **options)
        try:
            while process.poll() is None:
                rows = process_rows()
                selected = owned_rows(rows, process.pid, created_after)
                selected += [r for r in rows if r["pid"] in seen and r["born"] == seen[r["pid"]]["born"] and r not in selected]
                seen.update({r["pid"]: r for r in selected})
                peak = max(peak, sum(r["bytes"] for r in selected))
                if peak > limit_bytes or time.monotonic() - start > limit_seconds:
                    reason = "memory" if peak > limit_bytes else "timeout"
                    break
                time.sleep(1 if os.name == "nt" else 0.5)
            rows = process_rows()
            remaining = owned_rows(rows, process.pid, created_after)
            remaining += [r for r in rows if r["pid"] in seen and r["born"] == seen[r["pid"]]["born"] and r not in remaining]
            remaining = [r for r in remaining if r["pid"] != process.pid]
            if remaining and reason is None and process.returncode == 0 and settle_build_children:
                # Build tools may intentionally leave helpers alive. This is cleanup,
                # not an exemption: retain the evidence and require a fresh empty tree.
                before = remaining
                seen.update({r["pid"]: r for r in before})
                kill_owned(process, seen)
                rows = process_rows()
                remaining = owned_rows(rows, process.pid, created_after)
                remaining += [r for r in rows if r["pid"] in seen and r["born"] == seen[r["pid"]]["born"] and r not in remaining]
                remaining = [r for r in remaining if r["pid"] != process.pid]
                cleanup = {"before": before, "after": remaining, "settled": not remaining}
            if remaining and reason is None:
                reason = "unsettled child processes"
            if reason:
                seen.update({r["pid"]: r for r in remaining})
                kill_owned(process, seen)
        except BaseException:
            kill_owned(process, seen)
            raise
    return {"code": process.returncode, "stop": reason, "seconds": round(time.monotonic() - start, 3),
            "rootPid": process.pid, "root": seen.get(process.pid), "remaining": remaining, "buildCleanup": cleanup,
            "peakBytes": peak, "stdout": str(prefix.with_suffix(".out")), "stderr": str(prefix.with_suffix(".err"))}


def sha(path):
    with Path(path).open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def scope_deadline(started_at, wall_now, monotonic_now):
    # CI records the first step before checkout/setup; local runs start here.
    elapsed = 0.0
    if started_at is not None:
        try:
            started = float(started_at)
        except (ValueError, TypeError) as error:
            raise ValueError("Invalid platform start timestamp") from error
        if not math.isfinite(started) or started <= 0 or started > wall_now:
            raise ValueError("Invalid or future platform start timestamp")
        elapsed = wall_now - started
    remaining = 55 * 60 - elapsed
    if remaining <= 0:
        raise TimeoutError("Platform acceptance exhausted its 55-minute end-to-end budget during setup")
    return monotonic_now + remaining


def remaining_scope_seconds(deadline):
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError("Platform acceptance exhausted its 55-minute end-to-end budget")
    return min(1800, remaining)


def main():
    deadline = scope_deadline(os.environ.get("TSRUST_PLATFORM_STARTED_AT"), time.time(), time.monotonic())
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", required=True, choices=["x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-pc-windows-msvc"])
    parser.add_argument("--repair-plan", help="Prior run 42460717406 host artifact results JSON or directory; focused repair only")
    parser.add_argument("--second-repair-plan", help="Additional reviewed run 42470181942 artifact; requires first repair plan")
    parser.add_argument("--third-repair-plan", help="Additional reviewed run 42474075588 artifact; requires second repair plan")
    parser.add_argument("--later-repair-plan", action="append", default=[], help="Reviewed later runs in LATER_RUNS order; requires third repair plan")
    args = parser.parse_args()
    prior = repair_plan(args.repair_plan, args.host) if args.repair_plan else None
    assert not args.second_repair_plan or prior, "Second evidence requires first evidence"
    second = second_repair_plan(args.second_repair_plan, args.host) if args.second_repair_plan else None
    assert not args.third_repair_plan or second, "Third evidence requires second evidence"
    third = third_repair_plan(args.third_repair_plan, args.host) if args.third_repair_plan else None
    assert not args.later_repair_plan or third, "Later evidence requires third evidence"
    later = [later_repair_plan(path, args.host, index) for index, path in enumerate(args.later_repair_plan)]
    root = Path(__file__).resolve().parent.parent
    target = root / "target"
    version = tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    cargo = ["rustup", "run", version, "cargo"]
    suffix = ".exe" if os.name == "nt" else ""
    env = {**os.environ, "CARGO_TARGET_DIR": str(target), "TSRUST_CACHE_DIR": str(target / "tsrust-native"), "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "1",
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
    results.update(checkpoint=checkpoint, host=args.host, complete=False, scope="focused-repair" if prior else "full-native-family")
    if prior:
        results["retainedEvidence"] = {"checkpoint": REPAIR_CHECKPOINT, "resultsSha256": REPAIR_RESULTS[args.host], "run": "42460717406"}
        results["repairComplete"] = False
    def save():
        temporary = result_path.with_suffix(".tmp")
        temporary.write_text(json.dumps(results, indent=2) + "\n")
        temporary.replace(result_path)
    save()
    if second:
        results["secondRetainedEvidence"] = {"checkpoint": SECOND_CHECKPOINT, "resultsSha256": SECOND_RESULTS[args.host], "run": "42470181942"}
    if third:
        results["thirdRetainedEvidence"] = {"checkpoint": THIRD_CHECKPOINT, "resultsSha256": THIRD_RESULTS[args.host], "run": "42474075588"}
    if later:
        results["laterRetainedEvidence"] = earlier_seals(args.host, len(later))["laterRetainedEvidence"]
    def run(label, command, cwd=root):
        frozen()
        # Unique attempts preserve failed-scope evidence without overwriting logs.
        prefix = destination / (label + "-" + str(time.time_ns()))
        try:
            print("START", label, flush=True)
            row = bounded(command, cwd, env, prefix, limit_seconds=remaining_scope_seconds(deadline), settle_build_children=label == "selected-tests-build")
            print(json.dumps({"label": label, **{key: row[key] for key in ["code", "stop", "seconds", "peakBytes"]}}), flush=True)
            frozen()
        except BaseException as error:
            results.setdefault("attempts", []).append({"label": label, "stop": type(error).__name__, "message": str(error), "stdout": str(prefix.with_suffix(".out")), "stderr": str(prefix.with_suffix(".err"))})
            save()
            raise
        results.setdefault("attempts", []).append({"label": label, **row})
        save()
        if row["stop"]:
            raise RuntimeError(f"{label}: watchdog stopped: {row['stop']}")
        return row
    failures = []
    selected_names = sorted(set(COMPILER_TARGETS + LSP_TARGETS))
    row = run("selected-tests-build", cargo + ["test", "--offline", "--locked", "-p", "tsrust-compiler", "-p", "tsrust-lsp", "--features", "tsrust-compiler/http-websocket,tsrust-compiler/logging", *[argument for name in selected_names for argument in ["--test", name]], "--no-run", "--message-format=json"])
    if row["code"] != 0:
        results["failures"] = ["selected-tests::build"]
        save()
        raise RuntimeError("Selected test build failed")
    messages = [json.loads(line) for line in Path(row["stdout"]).read_text().splitlines() if line.startswith("{")]
    artifacts = selected_test_artifacts(messages, root)
    def consumers():
        return {name: sha(target / "debug" / (name + suffix)) for name in ["tsrust", "tsrust-lsp"]}
    row = run("portable", [str(target / "debug" / ("tsrust" + suffix)), "setup", "--portable", "--offline", "--yes"])
    assert row["code"] == 0, row
    for package, targets in [("tsrust-compiler", COMPILER_TARGETS), ("tsrust-lsp", LSP_TARGETS)]:
        for name in targets:
            key = package + "::" + name
            binary = artifacts[(package, name)]
            identity = {"checkpoint": checkpoint, "binary": sha(binary), "consumers": consumers()}
            previous = results.setdefault("targets", {}).get(key, {})
            tests = previous.get("tests", {}) if previous.get("identity") == identity else {}
            results["targets"][key] = {"identity": identity, "tests": tests}
            package_root = root / "crates" / package
            row = run(package + "-" + name + "-list", [str(binary), "--list", "--format", "terse"], package_root)
            assert row["code"] == 0, row
            names = [line[:-6] for line in Path(row["stdout"]).read_text().splitlines() if line.endswith(": test")]
            assert names and len(set(names)) == len(names), key
            retained = retained_cases(prior, key, names) if prior else set()
            if prior:
                results["targets"][key]["retained"] = {"identity": prior.get("targets", {}).get(key, {}).get("identity"), "cases": sorted(retained)}
            if second:
                second_cases = second_retained_cases(second, key, names) - retained
                results["targets"][key]["secondRetained"] = {"identity": second.get("targets", {}).get(key, {}).get("identity"), "cases": sorted(second_cases)}
                retained |= second_cases
            if third:
                third_cases = own_passes(third, key, names, THIRD_CHECKPOINT) - retained
                results["targets"][key]["thirdRetained"] = {"identity": third.get("targets", {}).get(key, {}).get("identity"), "cases": sorted(third_cases)}
                retained |= third_cases
            for index, evidence in enumerate(later):
                later_cases = own_passes(evidence, key, names, LATER_RUNS[index]["checkpoint"]) - retained
                results["targets"][key].setdefault("laterRetained", []).append({"run": LATER_RUNS[index]["run"], "identity": evidence.get("targets", {}).get(key, {}).get("identity"), "cases": sorted(later_cases)})
                retained |= later_cases
            pending = []
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
                if test in retained or tests.get(test, {}).get("success"):
                    continue
                pending.append(test)
            if pending:
                assert sha(binary) == identity["binary"] and consumers() == identity["consumers"]
                label = package + "-" + name + "-pending"
                command = [str(binary), "--exact", *pending, "--include-ignored", "--test-threads=3", "--format=pretty", "--color=never"]
                scope_error = None
                try:
                    row = run(label, command, package_root)
                except BaseException as error:
                    scope_error = error
                    row = results["attempts"][-1]
                    assert row["label"] == label
                assert sha(binary) == identity["binary"] and consumers() == identity["consumers"]
                output = Path(row["stdout"])
                text = output.read_text(errors="replace") if output.exists() else ""
                statuses, problems = target_statuses(text, pending, row.get("stop") is None and row.get("code") in [0, 101])
                valid_output = not any(problem.startswith(("Selected test count mismatch", "Unexpected test:", "Duplicate test:", "Test summary mismatch")) for problem in problems)
                if all(statuses.get(test) == "ok" for test in pending) and (row.get("code") != 0 or row.get("stop")):
                    valid_output = False
                for test in pending:
                    status = statuses.get(test, "missing")
                    success = valid_output and status == "ok" and row.get("stop") != "unsettled child processes"
                    tests[test] = {**row, "scopeCode": row.get("code"), "scopeStop": row.get("stop"), "code": 0 if success else row.get("code"), "status": status, "success": success}
                    print(key, test, "passed" if success else "FAILED", flush=True)
                    if not success:
                        failures.append(key + "::" + test)
                if problems or row.get("code") != 0 or row.get("stop"):
                    failures.append(key + "::pending-scope")
                results["failures"] = failures
                save()
                if scope_error:
                    raise scope_error
            if all(test in retained or tests.get(test, {}).get("success") or tests.get(test, {}).get("status") == "excluded" for test in names):
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
    results["complete"] = not bool(prior)
    if prior:
        results["repairComplete"] = True
    save()


if __name__ == "__main__":
    def interrupted(number, frame):
        raise SystemExit(128 + number)
    signal.signal(signal.SIGTERM, interrupted)
    main()
