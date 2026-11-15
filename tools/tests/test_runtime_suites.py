import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("runtime_suites", Path(__file__).parents[1] / "run-suites.py")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class RuntimeSuites(unittest.TestCase):
    def test_application_and_experimental_acceptance_is_local_only(self):
        groups = runner.suites(None)
        self.assertFalse(any(name.startswith(("apps/", "experiments/")) for name in groups))

    def test_library_and_example_suites_remain_registered(self):
        groups = runner.suites(None)
        self.assertIn("crates/tsrust-compiler/tests/fixtures/bytes", groups)
        self.assertIn("examples/testing", groups)
        files = [file for group in groups.values() for file in group]
        self.assertFalse(runner.EXPECTED_FAILURES.intersection(files))


if __name__ == "__main__":
    unittest.main()
