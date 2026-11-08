#!/usr/bin/env python3
"""Generate `tsrust test` suites from the editor catalog's verified examples.

Each non-UI catalog group becomes one directory, which `tsrust test` compiles into
one native program. Every example becomes a test that asserts its recorded
standard output through std:testing startOutputCapture/finishOutputCapture.

usage: generate-catalog-suites.py OUTPUT_DIRECTORY
"""
import json
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Examples that print from a spawned thread: output capture follows the test's
# own thread, so these stay with the threads host tests.
OTHER_THREAD_OUTPUT = {"std:threads.Channel #0"}
HEADER = "import { test as catalogTest, strictEqual as catalogEqual, startOutputCapture as catalogStart, finishOutputCapture as catalogFinish } from 'std:testing';\n"


def split_types(snippet):
    """Leading `type` declarations stay at module scope, as in the catalog runner."""
    prelude, snippet = [], snippet.lstrip()
    while snippet.startswith("type "):
        depth, end = 0, None
        for index, char in enumerate(snippet):
            if char in "{[(":
                depth += 1
            elif char in "}])":
                depth -= 1
            elif char == ";" and depth == 0:
                end = index + 1
                break
        prelude.append(snippet[:end])
        snippet = snippet[end:].lstrip()
    return "\n".join(prelude), snippet


def convert(source):
    """Return (module source, asynchronous) or None when the example needs a host test."""
    if "input.bin" in source:
        return None  # needs a prepared input file in the working directory
    if re.search(r"\b(currentExe|args)\s*\(", source):
        return None  # re-runs or inspects its own executable; a test runner is not that program
    if "DATABASE_URL" in source:
        return None  # needs a disposable external database
    if re.search(r"\bstdout\s*\(", source):
        return None  # writes raw process stdout, which console capture does not see
    if "std:testing" in source and re.search(r"\b(test|it|describe|suite|beforeAll|afterAll|beforeEach|afterEach|before|after|runTests)\s*\(", source):
        return None  # registers its own tests; the testing_library host tests cover it
    if re.search(r"\basync\s+function\s+main\s*\(", source):
        return re.sub(r"\basync\s+function\s+main\s*\(", "async function catalogProgram(", source, count=1), True
    if re.search(r"\bfunction\s+main\s*\(", source):
        return re.sub(r"\bfunction\s+main\s*\(", "function catalogProgram(", source, count=1), False
    prelude, body = split_types(source)
    if re.search(r"^\s*import\b", body, re.M):
        return None
    return f"{prelude}\nfunction catalogProgram(){{{body}}}", False


def quote(text):
    return json.dumps(text, ensure_ascii=False)


def main():
    output = Path(sys.argv[1])
    if output.exists():
        shutil.rmtree(output)
    catalog = json.loads((ROOT / "crates/tsrust-lsp/src/builtins.json").read_text())
    counts = {"tests": 0, "host": 0, "groups": 0}
    seen = set()
    for group, entries in sorted(catalog.items()):
        if group == "std:ui" or group.startswith(("Ui", "std:ui")):
            continue  # frozen UI
        directory = output / re.sub(r"[^A-Za-z0-9]+", "-", group).strip("-").lower()
        written = 0
        for entry in entries:
            for index, example in enumerate(entry.get("examples", [])):
                key = (example["source"], example["output"])
                if key in seen:
                    continue
                seen.add(key)
                converted = None if f"{group}.{entry['name']} #{index}" in OTHER_THREAD_OUTPUT else convert(example["source"])
                if converted is None:
                    counts["host"] += 1
                    continue
                module, asynchronous = converted
                name = f"{group}.{entry['name']} #{index}"
                if asynchronous:
                    check = f"catalogTest({quote(name)}, async () => {{ catalogStart(); await catalogProgram(); catalogEqual(catalogFinish(), {quote(example['output'])}); }});\n"
                else:
                    check = f"catalogTest({quote(name)}, () => {{ catalogStart(); catalogProgram(); catalogEqual(catalogFinish(), {quote(example['output'])}); }});\n"
                directory.mkdir(parents=True, exist_ok=True)
                slug = re.sub(r"[^A-Za-z0-9]+", "-", entry["name"]).strip("-").lower() or "entry"
                (directory / f"{slug}-{index}.test.tsr").write_text(HEADER + module + "\n" + check)
                written += 1
                counts["tests"] += 1
        counts["groups"] += bool(written)
    print(json.dumps(counts))


if __name__ == "__main__":
    main()
