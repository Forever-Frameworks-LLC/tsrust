"""Verify editor contracts when the compiler changes; state is only a build artifact."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('--force', action='store_true', help='Verify even when the compiler fingerprint is unchanged')
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
paths = sorted((root / 'crates/tsrust-compiler/src').glob('*.rs'))
paths += sorted((root / 'crates/tsrust-lsp/src').glob('*.rs'))
paths += sorted((root / 'crates/tsrust-lsp/tests').glob('*.rs'))
paths += [root / 'crates/tsrust-compiler/Cargo.toml', root / 'benchmarks/coverage/inventory.json', root / 'crates/tsrust-lsp/src/builtins.json', Path(__file__).resolve()]
hash_value = hashlib.sha256()
for path in paths:
    hash_value.update(str(path.relative_to(root)).encode())
    hash_value.update(path.read_bytes())
fingerprint = hash_value.hexdigest()
state = root / 'target/editor/compiler-compatibility.json'
previous = json.loads(state.read_text()) if state.exists() else {}
if not args.force and previous.get('fingerprint') == fingerprint:
    print('Compiler/editor contract unchanged; previous focused verification passed.')
else:
    environment = dict(os.environ, CARGO_TARGET_DIR=str(root / 'target/editor'))
    subprocess.run(['cargo', 'test', '--offline', '--locked', '-p', 'tsrust-lsp'], cwd=root, env=environment, check=True)
    catalog = json.loads((root / 'crates/tsrust-lsp/src/builtins.json').read_text())
    inventory = json.loads((root / 'benchmarks/coverage/inventory.json').read_text())
    additions = []
    for row in inventory['entries']:
        family, _, operation = row['id'].partition('.')
        catalog_family = {'String': 'string', 'Boolean': 'boolean', 'NumberConstructor': 'Number'}.get(family, 'number' if family == 'Number' and row.get('kind') == 'method' else family)
        if catalog_family in catalog and row.get('kind') in ['method', 'property'] and row['implementation'] in ['implemented', 'native variant']:
            if operation not in {entry['name'] for entry in catalog[catalog_family]}:
                additions.append(row['id'])
    latest = max((root / 'docs/milestones').glob('[0-9]*-*.md'), key=lambda p: int(p.name.split('-')[0]))
    report = {'fingerprint': fingerprint, 'latestMilestone': str(latest.relative_to(root)), 'verification': 'all focused tsrust-lsp contracts passed', 'catalogExpansionCandidates': additions}
    state.parent.mkdir(parents=True, exist_ok=True)
    state.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
