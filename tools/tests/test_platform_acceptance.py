import importlib.util
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
        for mode in ["async_acceptance", "toolchain_acceptance", "editor_acceptance", "full_regressions"]:
            self.assertFalse(enabled("platform", "platform_acceptance", extra=mode))
        platform = jobs["platform"]
        pairs = lambda source: re.findall(r"- os: (\S+)\s+target: (\S+)", source)
        self.assertEqual(pairs(platform), pairs((root / ".github/workflows/release.yml").read_text()))
        self.assertIn("persist-credentials: false", platform)
        self.assertNotIn("release.yml", platform)
        self.assertIn("PLATFORM_RESULT: ${{ needs.platform.result }}", jobs["ci"])


if __name__ == "__main__":
    unittest.main()
