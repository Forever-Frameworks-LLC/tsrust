import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("runtime_suites", Path(__file__).parents[1] / "run-suites.py")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class RuntimeSuites(unittest.TestCase):
    def test_application_and_experimental_acceptance_is_local_only(self):
        # Only the gallery (CI_APP_ROOTS) runs in CI; other applications stay local.
        groups = runner.suites(None)
        local = [name for name in groups if name.startswith(("apps/", "experiments/")) and not name.startswith(runner.CI_APP_ROOTS)]
        self.assertEqual(local, [])

    def test_library_and_example_suites_remain_registered(self):
        groups = runner.suites(None)
        self.assertIn("crates/tsrust-compiler/tests/fixtures/bytes", groups)
        self.assertIn("examples/testing", groups)
        files = [file for group in groups.values() for file in group]
        self.assertFalse(runner.EXPECTED_FAILURES.intersection(files))


if __name__ == "__main__":
    unittest.main()
