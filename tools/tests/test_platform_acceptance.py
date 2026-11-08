import ast
from contextlib import redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("platform_acceptance", Path(__file__).resolve().parents[1] / "platform-acceptance.py")
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


class PlatformAcceptance(unittest.TestCase):
    def test_target_statuses_keep_real_passes_and_reject_missing_duplicate_failed(self):
        complete = "running 2 tests\ntest a ... ok\ntest b ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n"
        self.assertEqual(RUNNER.target_statuses(complete, ["a", "b"], True), ({"a": "ok", "b": "ok"}, []))
        for text in [complete.replace("test b ... ok\n", ""), complete.replace("test b ... ok", "test a ... ok"), complete.replace("test b ... ok", "test b ... FAILED"), complete.replace("test b ... ok", "test b ... ignored"), complete.replace("running 2", "running 1"), complete.replace("test b ... ok", "test unknown ... ok")]:
            statuses, problems = RUNNER.target_statuses(text, ["a", "b"], True)
            self.assertTrue(problems)
        failed = complete.replace("test b ... ok", "test b ... FAILED").replace("test result: ok. 2 passed; 0 failed;", "test result: FAILED. 1 passed; 1 failed;")
        self.assertEqual(RUNNER.target_statuses(failed, ["a", "b"], True), ({"a": "ok", "b": "FAILED"}, ["b: FAILED"]))
        nested = failed.replace("test result: FAILED.", "failures:\n---- b stdout ----\nrunning 1 test\ntest nested ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n\ntest result: FAILED.")
        self.assertEqual(RUNNER.target_statuses(nested, ["a", "b"], True), ({"a": "ok", "b": "FAILED"}, ["b: FAILED"]))
        statuses, problems = RUNNER.target_statuses("running 2 tests\ntest a ... ok\n", ["a", "b"], False)
        self.assertEqual(statuses, {"a": "ok"})
        self.assertIn("b: missing", problems)
        source = (Path(__file__).resolve().parents[1] / "platform-acceptance.py").read_text()
        self.assertIn('"--exact", *pending, "--include-ignored", "--test-threads=3"', source)
        self.assertIn('"scopeCode": row.get("code")', source)
        self.assertIn('if scope_error:\n                    raise scope_error', source)

    def test_expected_panic_labels_preserve_exact_case_names_and_summary(self):
        text = "running 2 tests\ntest ordinary ... ok\ntest invariant - should panic ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored;\n"
        self.assertEqual(RUNNER.target_statuses(text, ["ordinary", "invariant"], True), ({"ordinary": "ok", "invariant": "ok"}, []))
        for changed in [text.replace("invariant - should panic", "unknown - should panic"), text.replace("invariant - should panic", "invariant - should panic - should panic"), text.replace("test invariant - should panic", "test ordinary ... ok\ntest invariant - should panic")]:
            self.assertTrue(RUNNER.target_statuses(changed, ["ordinary", "invariant"], True)[1])
        failed = text.replace("invariant - should panic ... ok", "invariant - should panic ... FAILED").replace("ok. 2 passed; 0 failed;", "FAILED. 1 passed; 1 failed;")
        self.assertEqual(RUNNER.target_statuses(failed, ["ordinary", "invariant"], True), ({"ordinary": "ok", "invariant": "FAILED"}, ["invariant: FAILED"]))

    def test_selected_build_keeps_exact_package_target_source_artifacts(self):
        root = Path(__file__).resolve().parents[2]
        messages = []
        expected = {(package, name) for package, names in [("tsrust-compiler", RUNNER.COMPILER_TARGETS), ("tsrust-lsp", RUNNER.LSP_TARGETS)] for name in names}
        self.assertEqual(len(expected), 46)
        files = {"std_fs", "native_file_cleanup", "native_full_files", "native_file_writes", "native_filesystem_utilities", "native_buffered_files", "native_buffered_file_backend"}
        self.assertTrue({("tsrust-compiler", name) for name in files} <= expected)
        selected = {(package, name) for package in ["tsrust-compiler", "tsrust-lsp"] for name in set(RUNNER.COMPILER_TARGETS + RUNNER.LSP_TARGETS) if (root / "crates" / package / "tests" / (name + ".rs")).is_file()}
        self.assertEqual(len(selected), 49)
        self.assertEqual(selected - expected, {("tsrust-compiler", "async_tasks"), ("tsrust-lsp", "http"), ("tsrust-lsp", "service_sections")})
        for package, name in sorted(selected):
            directory = root / "crates" / package
            version = RUNNER.tomllib.loads((directory / "Cargo.toml").read_text())["package"]["version"]
            messages.append({"reason": "compiler-artifact", "package_id": "path+" + directory.as_uri() + "#" + version, "manifest_path": str(directory / "Cargo.toml"), "target": {"name": name, "kind": ["test"], "test": True, "src_path": str(directory / "tests" / (name + ".rs"))}, "profile": {"test": True}, "executable": str(root / "target/debug/deps" / (package + "-" + name))})
        for package, name, source in [("tsrust-compiler", "tsrust", "src/cli/main.rs"), ("tsrust-lsp", "tsrust-lsp", "src/main.rs")]:
            directory = root / "crates" / package
            version = RUNNER.tomllib.loads((directory / "Cargo.toml").read_text())["package"]["version"]
            messages.append({"reason": "compiler-artifact", "package_id": "path+" + directory.as_uri() + "#" + version, "manifest_path": str(directory / "Cargo.toml"), "target": {"name": name, "kind": ["bin"], "src_path": str(directory / source)}, "profile": {"test": False}, "executable": str(root / "target/debug" / (name + (".exe" if os.name == "nt" else "")))})
        messages.append({"reason": "build-finished", "success": True})
        self.assertEqual(set(RUNNER.selected_test_artifacts(messages, root)), expected)
        for field, value in [("package_id", "registry+wrong#0.1.2"), ("manifest_path", "wrong/Cargo.toml"), ("executable", str(root / "other-target/test"))]:
            changed = [dict(row) for row in messages]
            changed[0][field] = value
            with self.assertRaises(AssertionError):
                RUNNER.selected_test_artifacts(changed, root)
        for changed in [messages[:-1], messages[1:], messages[:-1] + [messages[0], messages[-1]], messages[:-1] + [{"reason": "build-finished", "success": False}]]:
            with self.assertRaises(AssertionError):
                RUNNER.selected_test_artifacts(changed, root)
        changed = [dict(row) for row in messages]
        changed[0]["target"] = {**changed[0]["target"], "src_path": "wrong.rs"}
        with self.assertRaises(AssertionError):
            RUNNER.selected_test_artifacts(changed, root)
        changed[0] = {**messages[0], "target": {**messages[0]["target"], "name": "unknown_test"}}
        with self.assertRaises(AssertionError):
            RUNNER.selected_test_artifacts(changed, root)
        changed[0] = {**messages[0], "profile": {"test": False}}
        with self.assertRaises(AssertionError):
            RUNNER.selected_test_artifacts(changed, root)
        normal_index = next(index for index, row in enumerate(messages) if row.get("target", {}).get("name") == "tsrust")
        with self.assertRaises(AssertionError):
            RUNNER.selected_test_artifacts(messages[:normal_index] + messages[normal_index + 1:], root)
        for field, value in [("package_id", "wrong"), ("manifest_path", "wrong/Cargo.toml"), ("profile", {"test": True}), ("target", {**messages[normal_index]["target"], "src_path": "wrong.rs"}), ("executable", "wrong/tsrust")]:
            changed = [dict(row) for row in messages]
            changed[normal_index][field] = value
            with self.assertRaises(AssertionError):
                RUNNER.selected_test_artifacts(changed, root)
        source = (root / "tools/platform-acceptance.py").read_text()
        self.assertEqual(source.count('"--no-run"'), 1)
        self.assertIn('selected_names = sorted(set(COMPILER_TARGETS + LSP_TARGETS))', source)
        self.assertIn('binary = artifacts[(package, name)]', source)
        self.assertNotIn('"--tests"', source)
        self.assertNotIn('run("consumers"', source)
        self.assertLess(source.index('run("selected-tests-build"'), source.index('run("portable"'))

    def test_remaining_scope_budget_caps_shortens_and_rejects_exhaustion(self):
        with patch.object(RUNNER.time, "monotonic", side_effect=[100, 175, 200]):
            self.assertEqual(RUNNER.remaining_scope_seconds(5000), 1800)
            self.assertEqual(RUNNER.remaining_scope_seconds(200), 25)
            with self.assertRaisesRegex(TimeoutError, "55-minute end-to-end budget"):
                RUNNER.remaining_scope_seconds(200)

    def test_workflow_timeouts_never_exceed_one_hour(self):
        root = Path(__file__).resolve().parents[2]
        for path in (root / ".github/workflows").glob("*.yml"):
            for value in re.findall(r"timeout-minutes: (.+)", path.read_text()):
                self.assertTrue(value.isdigit() and 0 < int(value) <= 60, (path, value))

    def test_scope_progress_prints_flushed_start_and_compact_completion(self):
        source = (Path(__file__).resolve().parents[1] / "platform-acceptance.py").read_text()
        main = next(node for node in ast.parse(source).body if isinstance(node, ast.FunctionDef) and node.name == "main")
        function = next(node for node in main.body if isinstance(node, ast.FunctionDef) and node.name == "run")
        output = io.StringIO()
        row = {"code": 0, "stop": None, "seconds": 12.5, "peakBytes": 4096, "stdout": "scope.out", "stderr": "scope.err"}
        def bounded(*arguments, **options):
            self.assertEqual(output.getvalue(), "START fixture-build\n")
            self.assertEqual(options["limit_seconds"], 1800)
            return row
        namespace = {"root": Path("."), "destination": Path("."), "env": {}, "frozen": lambda: None, "save": lambda: None, "results": {}, "time": RUNNER.time, "json": json, "bounded": bounded, "remaining_scope_seconds": lambda deadline: 1800, "deadline": 3300}
        # Compile only the nested scope wrapper; main and real bounded/subprocess code cannot run.
        exec(compile(ast.Module(body=[function], type_ignores=[]), "scope-progress", "exec"), namespace)
        with redirect_stdout(output), patch("builtins.print", wraps=print) as printed:
            actual = namespace["run"]("fixture-build", ["never-executed"])
        self.assertIs(actual, row)
        self.assertEqual(printed.call_count, 2)
        self.assertTrue(all(call.kwargs.get("flush") is True for call in printed.call_args_list))
        lines = output.getvalue().splitlines()
        self.assertEqual(lines[0], "START fixture-build")
        self.assertEqual(json.loads(lines[1]), {"label": "fixture-build", "code": 0, "stop": None, "seconds": 12.5, "peakBytes": 4096})
        self.assertEqual(namespace["results"]["attempts"], [{"label": "fixture-build", **row}])

    def test_pinned_bun_rejects_corrupt_archive_before_execution(self):
        source = (Path(__file__).resolve().parents[2] / ".github/workflows/supporting-host-timing.yml").read_text()
        body = source.split("          python3 - <<'BUN_PYTHON'\n", 1)[1].split("          BUN_PYTHON", 1)[0]
        installer = compile("\n".join(line[10:] for line in body.splitlines()), "bun-installer", "exec")
        namespace = {}
        with patch("urllib.request.urlopen", return_value=io.BytesIO(b"x" * 37975495)), patch("subprocess.check_output") as execute:
            with self.assertRaises(AssertionError):
                exec(installer, namespace)
        execute.assert_not_called()
        self.assertNotIn("binary", namespace)

    def test_supporting_host_timing_is_reusable_bounded_and_uses_original_protocol(self):
        root = Path(__file__).resolve().parents[2]
        source = (root / ".github/workflows/supporting-host-timing.yml").read_text()
        self.assertIn("on:\n  workflow_call:\npermissions:", source)
        self.assertNotIn("pull_request:", source)
        self.assertIn("group: supporting-host-timing-helper-${{ github.ref }}", source)
        self.assertIn("cancel-in-progress: false", source)
        self.assertIn("runs-on: ubuntu-24.04", source)
        self.assertIn('limit_seconds=1800, limit_bytes=5_000_000 * 1024', source)
        self.assertIn('"--execution-only", "--engines=native,rust-hand,node-tsc", "--only=net_udp_large"', source)
        self.assertNotIn("--allow-busy", source)
        self.assertNotIn("oven-sh/setup-bun@", source)
        self.assertIn("bun-v1.3.11/bun-linux-x64-baseline.zip", source)
        self.assertIn("abe346f63414547cdf6b35b7a649a490c728b93d006226156923918a84c0e59b", source)
        self.assertLess(source.index("hashlib.sha256(payload).hexdigest()"), source.index("archive.read("))
        self.assertIn('messages[-1]["reason"] == "build-finished"', source)
        self.assertLess(source.index('run("portable"'), source.index('run("timing"'))
        self.assertLess(source.index('validate_result(data, expected)\n'), source.index('run("ingest"'))
        self.assertIn('path: ${{ steps.timing.outputs.result }}', source)
        body = source.split("          python3 - <<'PYTHON'\n", 1)[1].split("          PYTHON", 1)[0]
        parsed = ast.parse("\n".join(line[10:] for line in body.splitlines()))
        validator = next(node for node in ast.walk(parsed) if isinstance(node, ast.FunctionDef) and node.name == "validate_result")
        text = ast.unparse(validator)
        for guard in ["(3, 31, 8)", "len(rows) == 9", "== 93", "quiet['allowBusy'] is False", "row['checksum'] == expected", "keys =="]:
            self.assertIn(guard, text)

    def test_process_tree_includes_nested_and_regrouped_children_only(self):
        rows = [
            {"pid": 10, "ppid": 1, "pgid": 10},
            {"pid": 11, "ppid": 10, "pgid": 10},
            {"pid": 12, "ppid": 11, "pgid": 12},
            {"pid": 13, "ppid": 1, "pgid": 10},
            {"pid": 99, "ppid": 1, "pgid": 99},
        ]
        self.assertEqual({r["pid"] for r in RUNNER.owned_rows(rows, 10)}, {10, 11, 12, 13})

    def test_dead_windows_root_is_never_killed_by_a_reused_pid(self):
        class Finished:
            pid = 10
            def poll(self):
                return 0
            def wait(self, timeout):
                return 0
        current = [{"pid": 10, "born": "new"}, {"pid": 11, "born": "child"}]
        with patch.object(RUNNER.os, "name", "nt"), patch.object(RUNNER, "process_rows", return_value=current), patch.object(RUNNER.subprocess, "run") as run:
            RUNNER.kill_owned(Finished(), {10: {"born": "old"}, 11: {"born": "child"}})
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.args[0], ["taskkill", "/PID", "11", "/T", "/F"])

    def test_timeout_kills_and_joins_the_owned_child(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(RUNNER, "process_rows", return_value=[]):
            row = RUNNER.bounded([sys.executable, "-c", "import time; time.sleep(60)"], directory, os.environ.copy(), Path(directory) / "timeout", limit_seconds=0)
        self.assertEqual(row["stop"], "timeout")
        self.assertNotEqual(row["code"], 0)
        self.assertLess(row["seconds"], 5)

    def test_successful_root_with_remaining_child_keeps_failure_and_process_evidence(self):
        class Finished:
            pid = 10
            returncode = 0
            def poll(self):
                return 0
        child = {"pid": 11, "ppid": 10, "born": "2026-11-06T17:00:00.1234567Z", "name": "worker.exe", "bytes": 4096}
        with tempfile.TemporaryDirectory() as directory, patch.object(RUNNER.subprocess, "Popen", return_value=Finished()), patch.object(RUNNER, "process_rows", return_value=[child]), patch.object(RUNNER, "kill_owned") as cleanup:
            row = RUNNER.bounded(["finished"], directory, os.environ.copy(), Path(directory) / "remaining")
        self.assertEqual(row["code"], 0)
        self.assertEqual(row["stop"], "unsettled child processes")
        self.assertEqual(row["rootPid"], 10)
        self.assertEqual(row["remaining"], [child])
        cleanup.assert_called_once()

    def test_counter_failure_still_kills_and_joins_child(self):
        process = None
        original = subprocess.Popen
        def started(*args, **kwargs):
            nonlocal process
            process = original(*args, **kwargs)
            return process
        with tempfile.TemporaryDirectory() as directory, patch.object(RUNNER.subprocess, "Popen", side_effect=started), patch.object(RUNNER, "process_rows", side_effect=RuntimeError("counter unavailable")):
            with self.assertRaisesRegex(RuntimeError, "counter unavailable"):
                RUNNER.bounded([sys.executable, "-c", "import time; time.sleep(60)"], directory, os.environ.copy(), Path(directory) / "counter")
        # POSIX group termination precedes the failed secondary counter. Reap it.
        process.wait(timeout=5)
        self.assertIsNotNone(process.returncode)

    def test_selected_test_sources_and_verified_tls_fixture_exist(self):
        root = Path(__file__).resolve().parents[2]
        for package, targets in [("tsrust-compiler", RUNNER.COMPILER_TARGETS), ("tsrust-lsp", RUNNER.LSP_TARGETS)]:
            for target in targets:
                self.assertTrue((root / "crates" / package / "tests" / (target + ".rs")).is_file(), target)
        backend = (root / "crates/tsrust-compiler/tests/native_buffered_file_backend.rs").read_text()
        self.assertIn('include_str!("../src/stdlib/fs/runtime/fs.rs")', backend)
        self.assertIn('"--test"', backend)
        self.assertIn('for level in ["0","3"]', backend)
        self.assertIn("fn windows_close_and_scope_drop_release_exclusive_open_conflicts", (root / "crates/tsrust-compiler/src/stdlib/fs/runtime/fs.rs").read_text())
        self.assertIn("fn https_example_executes_with_verified_private_ca", (root / "crates/tsrust-compiler/tests/http.rs").read_text())
        self.assertIn("fn udp_large_datagram_delivers_all_60000_bytes", (root / "crates/tsrust-compiler/tests/network_transport_runtime.rs").read_text())

    def test_platform_job_is_manual_exclusive_and_matches_release_hosts(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/ci.yml").read_text()
        jobs = dict(re.findall(r"^  (\w+):\n(.*?)(?=^  \w+:\n|\Z)", workflow.split("jobs:\n", 1)[1], re.M | re.S))
        conditions = {name: re.search(r"^    if: \$\{\{ (.*?) \}\}$", source, re.M).group(1) for name, source in jobs.items() if name != "ci"}
        def enabled(name, mode=None, event="workflow_dispatch", extra=None):
            expression = conditions[name].replace("github.event_name", repr(event))
            expression = re.sub(r"inputs\.(\w+)", lambda match: str(match[1] in {mode, extra}), expression)
            return eval(expression.replace("&&", " and ").replace("||", " or ").replace("!", "not ").strip(), {"__builtins__": {}})
        self.assertEqual({name for name in conditions if enabled(name)}, {"source", "editor"})
        self.assertEqual({name for name in conditions if enabled(name, "platform_acceptance")}, {"platform"})
        self.assertFalse(enabled("platform", "platform_acceptance", "pull_request"))
        self.assertEqual({name for name in conditions if enabled(name, "supporting_host_timing")}, {"supporting_host_timing"})
        self.assertFalse(enabled("supporting_host_timing", "supporting_host_timing", "pull_request"))
        for mode in ["platform_acceptance", "async_acceptance", "toolchain_acceptance", "editor_acceptance", "full_regressions"]:
            self.assertFalse(enabled("supporting_host_timing", "supporting_host_timing", extra=mode))
        self.assertIn("uses: ./.github/workflows/supporting-host-timing.yml", jobs["supporting_host_timing"])
        self.assertIn("supporting-host-timing-caller", workflow)
        self.assertIn("SUPPORTING_HOST_RESULT: ${{ needs.supporting_host_timing.result }}", jobs["ci"])
        for mode in ["async_acceptance", "toolchain_acceptance", "editor_acceptance", "full_regressions", "supporting_host_timing"]:
            self.assertFalse(enabled("platform", "platform_acceptance", extra=mode))
        platform = jobs["platform"]
        pairs = lambda source: re.findall(r"- os: (\S+)\s+target: (\S+)", source)
        selections = re.search(r"inputs\.platform_windows_only &&\s*'([^']+)' \|\|\s*'([^']+)'", platform)
        self.assertIsNotNone(selections)
        windows, all_hosts = ([(row["os"], row["target"]) for row in json.loads(value)] for value in selections.groups())
        self.assertEqual(all_hosts, pairs((root / ".github/workflows/release.yml").read_text()))
        self.assertEqual(windows, [("windows-2022", "x86_64-pc-windows-msvc")])
        self.assertIn("persist-credentials: false", platform)
        self.assertNotIn("release.yml", platform)
        self.assertIn("PLATFORM_RESULT: ${{ needs.platform.result }}", jobs["ci"])


if __name__ == "__main__":
    unittest.main()
