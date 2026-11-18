import runpy
import unittest
from pathlib import Path

runner = runpy.run_path(str(Path(__file__).resolve().parents[1] / "run-suites.py"))


class SuiteRunnerTests(unittest.TestCase):
    def test_only_gpui_starter_has_a_cold_native_build_budget(self):
        self.assertEqual(runner["suite_timeout"](Path("apps/gpui-starter/tests")), 1800)
        for directory in ["examples/testing", "target/catalog-suites/std-ui", "apps/gpui-starter/tests-other"]:
            self.assertEqual(runner["suite_timeout"](directory), 600)

    def test_gallery_app_stays_in_ci_while_other_apps_stay_local(self):
        self.assertTrue(runner["ci_suite"]("apps/gpui-starter/tests/pages.test.tsr"))
        self.assertTrue(runner["ci_suite"]("examples/testing/basics.test.tsr"))
        for file in ["apps/studio/tests/git.test.tsr", "apps/tasks/desktop/tests/service.test.tsr", "experiments/x/a.test.tsr"]:
            self.assertFalse(runner["ci_suite"](file))

    def test_build_and_test_profiles_stay_separate(self):
        output = '\n'.join(['console output', '{invalid', '[]',
                            '{"event":"build","durationMs":700123.4}',
                            '{"event":"test","durationMs":5}',
                            '{"event":"summary","durationMs":12.5}'])
        self.assertEqual(runner["profile_times"](output), {"buildSeconds": 700.123, "testSeconds": 0.013})
        self.assertEqual(runner["profile_times"]("compile failure"), {})
