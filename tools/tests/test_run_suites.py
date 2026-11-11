import runpy
import unittest
from pathlib import Path

runner = runpy.run_path(str(Path(__file__).resolve().parents[1] / "run-suites.py"))


class SuiteRunnerTests(unittest.TestCase):
    def test_only_gpui_starter_has_a_cold_native_build_budget(self):
        self.assertEqual(runner["suite_timeout"](Path("examples/gpui-starter/tests")), 1800)
        for directory in ["examples/testing", "target/catalog-suites/std-ui", "examples/gpui-starter/tests-other"]:
            self.assertEqual(runner["suite_timeout"](directory), 600)

    def test_build_and_test_profiles_stay_separate(self):
        output = '\n'.join(['console output', '{invalid', '[]',
                            '{"event":"build","durationMs":700123.4}',
                            '{"event":"test","durationMs":5}',
                            '{"event":"summary","durationMs":12.5}'])
        self.assertEqual(runner["profile_times"](output), {"buildSeconds": 700.123, "testSeconds": 0.013})
        self.assertEqual(runner["profile_times"]("compile failure"), {})
