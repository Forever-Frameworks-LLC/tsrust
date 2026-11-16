#!/usr/bin/env python3
"""Estimate how a generated Tsrust program could split into Rust crates.

Rust crates cannot depend on each other cyclically, so items that reference
each other in a cycle must share a crate. This tool reads a program emitted
with `tsrust emit --source-locations`, groups every top-level item with the
impl blocks of the type it defines, builds the reference graph between those
groups, condenses it into strongly connected components (SCCs) and reports:

- the size of the largest SCC (code that must compile as one crate),
- for each source file, how much generated code an ideal split by SCC would
  rebuild after an edit to that file (its SCCs plus everything depending on
  them), compared with the whole program.

usage: crate-split-analysis.py PROGRAM.rs [--json] [--without REGEX]

--without REGEX treats matching groups as if they no longer referenced other
groups (for example, stored-callable families dispatching through a trait
object instead of an enum over concrete closures) to show which
representations hold a cycle together.
"""
import collections
import json
import re
import sys

KEYWORD = re.compile(r"^(?:pub(?:\([a-z]+\))? )?(?:async |unsafe |extern \"[^\"]*\" )?(fn|struct|enum|union|type|trait|mod|const|static|impl|use|macro_rules!)\b")
IDENT = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")
POSITION = re.compile(r'// tsrust-position \{"path":"((?:[^"\\]|\\.)*)"')


def top_level_lines(data):
    """Byte offsets of lines starting outside delimiters, strings and comments."""
    n = len(data)
    starts = [0]
    depth = 0
    i = 0

    def ident(c):
        return chr(c).isalnum() or c == 95 or c >= 0x80

    def skip_string(i):
        while i < n:
            if data[i] == 92:
                i += 2
            elif data[i] == 34:
                return i + 1
            else:
                i += 1
        return n

    def prefixed(start):
        i = start
        if data[i] in (98, 99):
            i += 1
            if i < n and data[i] == 34:
                return skip_string(i + 1)
            if i < n and data[i] == 39 and data[start] == 98:
                e = i + 1
                if e < n and data[e] == 92:
                    e += 1
                e += 1
                while e < n and data[e] != 39:
                    e += 1
                return e + 1
            if not (i < n and data[i] == 114):
                return None
        if not (i < n and data[i] == 114):
            return None
        i += 1
        hashes = 0
        while i < n and data[i] == 35:
            hashes += 1
            i += 1
        if not (i < n and data[i] == 34):
            return None
        i += 1
        while True:
            q = data.find(b'"', i)
            if q < 0:
                return n
            if data[q + 1:q + 1 + hashes] == b"#" * hashes:
                return q + 1 + hashes
            i = q + 1

    while i < n:
        c = data[i]
        if c == 10:
            if depth == 0 and i + 1 < n:
                starts.append(i + 1)
            i += 1
        elif c in b"{([":
            depth += 1
            i += 1
        elif c in b"})]":
            depth -= 1
            i += 1
        elif c == 47 and i + 1 < n and data[i + 1] == 47:
            k = data.find(b"\n", i)
            i = n if k < 0 else k
        elif c == 47 and i + 1 < n and data[i + 1] == 42:
            nest = 1
            i += 2
            while nest > 0 and i < n:
                if data[i:i + 2] == b"/*":
                    nest += 1
                    i += 2
                elif data[i:i + 2] == b"*/":
                    nest -= 1
                    i += 2
                else:
                    i += 1
        elif c == 34:
            i = skip_string(i + 1)
        elif c in (114, 98, 99) and not ident(data[i - 1] if i else 32):
            r = prefixed(i)
            i = r if r is not None else i + 1
        elif c == 39:
            j = i + 1
            if data[j] == 92:
                i = data.find(b"'", j + 2) + 1
            else:
                lead = data[j]
                width = 1 if lead < 0x80 else 2 if lead < 0xE0 else 3 if lead < 0xF0 else 4
                i = j + width + 1 if j + width < n and data[j + width] == 39 else i + 1
        else:
            i += 1
    return starts


def items(text):
    data = text.encode()
    starts = top_level_lines(data)
    heads = [s for s in starts if KEYWORD.match(data[s:s + 120].decode(errors="ignore"))]
    heads.append(len(data))
    result = []
    for a, b in zip(heads, heads[1:]):
        result.append(data[a:b].decode(errors="ignore"))
    return result


def owner(item):
    head = item.split("\n", 1)[0]
    kind = KEYWORD.match(head).group(1)
    if kind == "impl":
        # impl<...> Trait for Type<...> / impl<...> Type<...>
        rest = re.sub(r"^(?:unsafe )?impl\s*", "", head)
        if rest.startswith("<"):
            depth = 0
            for index, char in enumerate(rest):
                depth += char == "<"
                depth -= char == ">"
                if depth == 0:
                    rest = rest[index + 1:]
                    break
        target = rest.split(" for ", 1)[1] if " for " in rest.split("{", 1)[0] else rest
        name = IDENT.search(target.replace("std::", "").replace("&", " ").replace("'a", " "))
        return kind, name.group(0) if name else None
    if kind in ("use", "macro_rules!"):
        return kind, None
    name = re.search(r"\b(?:fn|struct|enum|union|type|trait|mod|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)", head)
    return kind, name.group(1) if name else None


def strongly_connected(nodes, edges):
    index = {}
    low = {}
    stack = []
    on_stack = set()
    components = []
    counter = 0
    for root in nodes:
        if root in index:
            continue
        work = [(root, iter(edges[root]))]
        index[root] = low[root] = counter
        counter += 1
        stack.append(root)
        on_stack.add(root)
        while work:
            node, children = work[-1]
            advanced = False
            for child in children:
                if child not in index:
                    index[child] = low[child] = counter
                    counter += 1
                    stack.append(child)
                    on_stack.add(child)
                    work.append((child, iter(edges[child])))
                    advanced = True
                    break
                if child in on_stack:
                    low[node] = min(low[node], index[child])
            if advanced:
                continue
            work.pop()
            if work:
                low[work[-1][0]] = min(low[work[-1][0]], low[node])
            if low[node] == index[node]:
                component = []
                while True:
                    member = stack.pop()
                    on_stack.discard(member)
                    component.append(member)
                    if member == node:
                        break
                components.append(component)
    return components


def main():
    path = sys.argv[1]
    text = open(path, encoding="utf-8").read()
    groups = collections.defaultdict(list)
    shared = []
    for item in items(text):
        kind, name = owner(item)
        if name is None:
            shared.append(item)
        else:
            groups[name].append(item)
    names = set(groups)
    size = {name: sum(len(item) for item in body) for name, body in groups.items()}
    files = {}
    edges = {}
    for name, body in groups.items():
        joined = "\n".join(body)
        found = POSITION.findall(joined)
        if found:
            files[name] = collections.Counter(found).most_common(1)[0][0]
        edges[name] = sorted({token for token in IDENT.findall(joined) if token in names and token != name})
    if "--without" in sys.argv:
        pattern = re.compile(sys.argv[sys.argv.index("--without") + 1])
        for name in edges:
            if pattern.fullmatch(name):
                edges[name] = []
    components = strongly_connected(sorted(names), edges)
    component_of = {member: index for index, component in enumerate(components) for member in component}
    component_size = [sum(size[member] for member in component) for component in components]
    dependents = collections.defaultdict(set)
    for name, targets in edges.items():
        for target in targets:
            if component_of[target] != component_of[name]:
                dependents[component_of[target]].add(component_of[name])
    total = sum(size.values())
    largest = max(range(len(components)), key=lambda index: component_size[index])
    largest_files = collections.Counter(files[m] for m in components[largest] if m in files)

    def rebuild(component_ids):
        seen = set(component_ids)
        pending = list(component_ids)
        while pending:
            for parent in dependents[pending.pop()]:
                if parent not in seen:
                    seen.add(parent)
                    pending.append(parent)
        return sum(component_size[index] for index in seen)

    per_file = {}
    by_file = collections.defaultdict(set)
    for name, file in files.items():
        by_file[file].add(component_of[name])
    for file, component_ids in by_file.items():
        per_file[file] = rebuild(component_ids)
    report = {
        "itemBytes": total,
        "sharedUnownedBytes": sum(len(item) for item in shared),
        "groups": len(groups),
        "components": len(components),
        "largestComponentBytes": component_size[largest],
        "largestComponentShare": round(component_size[largest] / total, 3),
        "largestComponentFiles": len(largest_files),
        "medianRebuildShare": round(sorted(per_file.values())[len(per_file) // 2] / total, 3) if per_file else None,
        "worstRebuildShare": round(max(per_file.values()) / total, 3) if per_file else None,
        "files": len(per_file),
    }
    if "--json" in sys.argv:
        print(json.dumps(report, indent=2))
        return
    for key, value in report.items():
        print(f"{key}: {value}")
    print("largest component members by kind:")
    kinds = collections.Counter(re.sub(r"\d+", "N", member) for member in components[largest])
    for kind, count in kinds.most_common(12):
        print(f"  {count:6} {kind}")


if __name__ == "__main__":
    main()
