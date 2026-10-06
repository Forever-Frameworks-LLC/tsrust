#!/usr/bin/env python3
"""Fail on retired type spellings in live sources; historical evidence is excluded."""
from pathlib import Path
import re
import sys
from migrate_function_types import convert

ROOT = Path(__file__).resolve().parents[1]
# Internal access enums/analysis JSON and ordinary identifiers are not source syntax.
# Keyword-rejection cases intentionally construct their source at runtime.
DIRECT = re.compile(r'\b(?:owned\s+once|owned|once|mutate|read)\s*\([^\n]*?=>|:\s*(?:read|mutate|consume)\s+(?:string|int|number|boolean|[A-Z][A-Za-z0-9_]*)\b')
findings = []
for directory in ['crates/tsrust-compiler/src', 'crates/tsrust-compiler/tests', 'crates/tsrust-lsp/src', 'crates/tsrust-lsp/tests', 'editors/vscode/src', 'editors/vscode/snippets', 'benchmarks/runtime/workloads']:
    for path in sorted((ROOT / directory).rglob('*')):
        if path.suffix not in ['.rs', '.tsr', '.ts', '.json']:
            continue
        text = path.read_text()
        for number, line in enumerate(text.splitlines(), 1):
            if DIRECT.search(line) and convert(line) != line:
                findings.append(f'{path.relative_to(ROOT)}:{number}: {line.strip()[:180]}')
if findings:
    print('\n'.join(findings))
    sys.exit(1)
print('No retired type spellings in live compiler/editor/fixture/workload sources.')
