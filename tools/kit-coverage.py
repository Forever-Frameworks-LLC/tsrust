#!/usr/bin/env python3
"""Report how much of GPUI Kit's public component API `std:ui` covers.

Inventories every public method on GPUI Kit's renderable component types and
their state types (gpui-component, plus the gpui-base types it re-exports) from
the locked crate sources, then checks each against the `covers` map in
stdlib/ui/kit.json:

    "covers": {"Button": {"primary": "variant", "on_click": "onPress", "id": "n/a: automatic"}}

A method is covered when it maps to a prop, event or child, or carries an
explicit "n/a: reason". Exit status is nonzero while anything is unmapped, so
`--check` can gate completeness. `--component Name` lists one type's gaps.
"""
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "crates/tsrust-compiler/src/stdlib/ui/kit.json"
REGISTRY = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")) / "registry/src"
TRAITS = {"Sizable": ["xsmall", "small", "medium", "large", "with_size"], "Disableable": ["disabled"], "Selectable": ["selected"],
          "ButtonVariants": ["primary", "secondary", "danger", "warning", "success", "info", "ghost", "link", "text", "custom"],
          "Collapsible": ["collapsed"]}


def crate(name):
    matches = sorted(REGISTRY.glob(f"*/{name}"))
    if not matches:
        sys.exit(f"{name} sources are not in the Cargo registry; run tsrust setup --dependencies --yes")
    return matches[-1] / "src"


def block_end(text, index):
    """Index just past the brace block whose opening brace precedes `index`, skipping
    braces inside comments, string, raw string and char literals."""
    depth = 1
    length = len(text)
    while depth and index < length:
        char = text[index]
        if text.startswith("//", index):
            index = text.find("\n", index)
            index = length if index < 0 else index
            continue
        if text.startswith("/*", index):
            index = text.find("*/", index + 2)
            index = length if index < 0 else index + 2
            continue
        raw = re.match(r'b?r(#*)"', text[index:index + 12]) if char in "rb" and not (text[index - 1].isalnum() or text[index - 1] == "_") else None
        if raw:
            close = '"' + raw.group(1)
            index = text.find(close, index + raw.end())
            index = length if index < 0 else index + len(close)
            continue
        if char == '"':
            index += 1
            while index < length and text[index] != '"':
                index += 2 if text[index] == "\\" else 1
            index += 1
            continue
        if char == "'":
            literal = re.match(r"'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^\\'])'", text[index:index + 12])
            if literal:
                index += literal.end()
                continue
        depth += {"{": 1, "}": -1}.get(char, 0)
        index += 1
    return index


def impls(text):
    """Yield (header, body) for every impl block, matching braces."""
    for match in re.finditer(r"^impl\b[^{;]*\{", text, re.M):
        yield match.group(0)[:-1], text[match.end():block_end(text, match.end())]


def without_tests(text):
    """Drop `#[cfg(test)] mod name { ... }` blocks, matching braces."""
    while (match := re.search(r"#\[cfg\(test\)\]\s*mod \w+\s*\{", text)):
        text = text[:match.start()] + text[block_end(text, match.end()):]
    return text


def target(header):
    """The type an impl header targets, and the trait when it is a trait impl."""
    header = re.sub(r"\s+", " ", header.split(" where ")[0])
    rest = header[4:].lstrip()
    if rest.startswith("<"):
        depth = 0
        for index, char in enumerate(rest):
            depth += {"<": 1, ">": -1}.get(char, 0)
            if depth == 0:
                rest = rest[index + 1:].lstrip()
                break
    if " for " in rest:
        trait, ty = rest.split(" for ", 1)
        name = re.search(r"\b([A-Z]\w*)", ty.split("<")[0])
        return (name.group(1) if name else None), re.sub(r"<.*", "", trait).split("::")[-1].strip()
    found = re.match(r"(?:\w+::)*([A-Z]\w*)", rest)
    return (found.group(1) if found else None), None


def inventory():
    types = {}
    for source in [crate("gpui-component-0.9.0"), crate("gpui-base-0.9.0")]:
        for path in source.rglob("*.rs"):
            if "test" in path.name:
                continue
            text = without_tests(path.read_text())
            for header, body in impls(text):
                ty, trait = target(header)
                if not ty:
                    continue
                # A gpui-component type shadows a gpui-base type of the same name.
                if source.parent.name.startswith("gpui-base") and any(f.startswith("gpui-component") for f in types.get(ty, {}).get("files", ())):
                    continue
                # Mode-specific impls of the generic input state belong to their aliases.
                if ty == "InputBaseState" and "MultiLineMode" in header:
                    ty = "InputBaseState:multi"
                elif ty == "InputBaseState" and (mode := re.search(r"InputBaseState<(?:crate::input::)?(Input|Textarea|Editor)Mode>", header)):
                    ty = mode.group(1) + "State"
                entry = types.setdefault(ty, {"methods": [], "traits": set(), "files": set()})
                entry["files"].add(str(path.relative_to(source.parent.parent)))
                if trait:
                    entry["traits"].add(trait)
                else:
                    for method in re.findall(r"^\s{4}pub fn (\w+)", body, re.M):
                        if method not in entry["methods"]:
                            entry["methods"].append(method)
    # Aliases such as `pub type InputState = InputBaseState<InputMode>` expose the base type's methods.
    for source in [crate("gpui-component-0.9.0"), crate("gpui-base-0.9.0")]:
        for path in source.rglob("*.rs"):
            for alias, base in re.findall(r"^pub type (\w+) = (\w+)<", path.read_text(), re.M):
                if base not in types:
                    continue
                entry = types.setdefault(alias, {"methods": [], "traits": set(), "files": set()})
                inherited = types[base]["methods"] + (types.get(base + ":multi", {}).get("methods", []) if alias in ("TextareaState", "EditorState") else [])
                entry["methods"] += [method for method in inherited if method not in entry["methods"]]
                entry["traits"] |= types[base]["traits"]
    for entry in types.values():
        for trait, methods in TRAITS.items():
            if trait in entry["traits"]:
                entry["methods"] += [method for method in methods if method not in entry["methods"]]
    return types


def main():
    spec = json.loads(SPEC.read_text())
    covers = spec.get("covers", {})
    types = inventory()
    tracked = {name: types[name] for name in spec.get("kitTypes", []) if name in types}
    missing = [name for name in spec.get("kitTypes", []) if name not in types]
    only = sys.argv[sys.argv.index("--component") + 1] if "--component" in sys.argv else None
    total = done = 0
    gaps = {}
    for name, entry in sorted(tracked.items()):
        mapped = covers.get(name, {})
        open_methods = [method for method in entry["methods"] if method not in mapped]
        total += len(entry["methods"])
        done += len(entry["methods"]) - len(open_methods)
        if open_methods:
            gaps[name] = open_methods
    if only:
        print(f"{only}: {', '.join(gaps.get(only, [])) or 'complete'}")
    else:
        for name, methods in gaps.items():
            print(f"{name} ({len(methods)}): {', '.join(methods)}")
        print(f"Covered {done}/{total} public methods across {len(tracked)} GPUI Kit types; {len(gaps)} types incomplete.")
    if missing:
        print("Unknown kitTypes:", ", ".join(missing))
    if "--check" in sys.argv and (gaps or missing):
        sys.exit(1)


if __name__ == "__main__":
    main()
