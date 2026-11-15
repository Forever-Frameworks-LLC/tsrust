import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class ApiReferenceTest(unittest.TestCase):
    def test_catalog_parameter_records_have_editor_labels(self):
        catalog = json.loads((ROOT / 'crates/tsrust-lsp/src/builtins.json').read_text())
        for group, entries in catalog.items():
            for entry in entries:
                for parameter in entry.get('parameters') or []:
                    if isinstance(parameter, dict):
                        self.assertTrue(parameter.get('label'), (group, entry['name'], parameter))

    def test_ui_reference_includes_signal_methods_and_completion_parameters(self):
        page = (ROOT / 'website/src/content/docs/reference/library/ui.md').read_text()
        for name in ['Signal.get', 'Signal.set', 'Signal.handle']:
            self.assertIn('### ' + name, page)
        completion = page.split('### completionText\n', 1)[1].split('\n### ', 1)[0]
        for label in ['text: string', 'item: string', 'start: int', 'end: int']:
            self.assertIn(label, completion)


if __name__ == '__main__':
    unittest.main()
