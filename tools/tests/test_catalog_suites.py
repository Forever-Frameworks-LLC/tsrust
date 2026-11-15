import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


class CatalogSuitesTest(unittest.TestCase):
    def test_case_distinct_groups_and_members_keep_every_example(self):
        spec = importlib.util.spec_from_file_location("catalog_suites", Path(__file__).resolve().parents[1] / "generate-catalog-suites.py")
        generator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(generator)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            catalog = root / "crates/tsrust-lsp/src/builtins.json"
            catalog.parent.mkdir(parents=True)
            def entry(name, value):
                return {"name": name, "examples": [{"source": f"console.log({value});", "output": f"{value}\n"}]}
            catalog.write_text(json.dumps({"Number": [entry("Request", 1), entry("request", 2)], "number": [entry("value", 3)]}))
            output = root / "suites"
            report = io.StringIO()
            with patch.object(generator, "ROOT", root), patch("sys.argv", ["generate-catalog-suites.py", str(output)]), contextlib.redirect_stdout(report):
                generator.main()
            files = list(output.glob("*/*.test.tsr"))
            self.assertEqual(json.loads(report.getvalue()), {"tests": 3, "host": 0, "groups": 2})
            self.assertEqual(len(files), 3)
            self.assertEqual(len({str(path.relative_to(output)).casefold() for path in files}), 3)
            self.assertEqual(len({path.parent for path in files}), 2)


if __name__ == "__main__":
    unittest.main()
