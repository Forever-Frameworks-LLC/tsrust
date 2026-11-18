#!/usr/bin/env python3
"""Prototype: split a generated Tsrust program into one crate per source module.

usage: split_modules.py PROGRAM.rs CRATE_PLAN.json OUT_DIR

PROGRAM.rs is plain `tsrust emit` output (numbers as created, so they index the
`--crate-plan` arrays). Writes OUT_DIR/tsrust_types.rs, OUT_DIR/tsrust_m<K>.rs,
OUT_DIR/program.rs and OUT_DIR/crates.json (build order and dependencies).

Crates:
- tsrust_types: every item used from more than one crate (records, unions,
  families, shared closures, runtime glue) and compiler-made functions.
- tsrust_m<K>: generated functions of source module K, closures and types only
  that module uses, and bodies of its synchronous closures that had to move to
  tsrust_types (called through a link-time declaration).
- program: `main` and a compile-time check of every link-time declaration.

Calls between crates go through `unsafe extern "Rust"` declarations resolved by
the linker to `#[unsafe(export_name)]` definitions: direct calls, unchanged
code. Module crates depend only on tsrust_types, so they build in parallel.
"""
import collections
import importlib.util
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("analysis", os.path.join(HERE, "../../tools/crate-split-analysis.py"))
analysis = importlib.util.module_from_spec(spec)
spec.loader.exec_module(analysis)

TYPES = "tsrust_types"
PROGRAM = "program"
KEYWORD = re.compile(r"^(?:#\[[^\n]*?\] )*(?:pub(?:\([a-z]+\))? )?(?:async |unsafe |extern \"[^\"]*\" )*(fn|struct|enum|union|type|trait|mod|const|static|impl|use|macro_rules!)\b")
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
FUNCTION = re.compile(r"(?:tsr_result_function|function)(\d+)")


def split_items(text):
    """Top-level items with their leading attribute lines; inner attributes and comments stand alone."""
    data = text.encode()
    starts = analysis.top_level_lines(data)
    starts.append(len(data))
    items = []
    pending = None
    for a, b in zip(starts, starts[1:]):
        line = data[a:b].decode()
        stripped = line.strip()
        if stripped.startswith("#[") and stripped.endswith("]") and not KEYWORD.match(line):
            pending = line if pending is None else pending + line
            continue
        if KEYWORD.match(line) or not items or stripped.startswith("#!") or (stripped.startswith("//") and pending is None):
            items.append((pending or "") + line)
            pending = None
        else:
            items[-1] += (pending or "") + line
            pending = None
    if pending:
        items.append(pending)
    return items


def closes_at_depth_zero(text):
    """Offsets just after a `}` that returns to depth zero, outside literals and comments."""
    out = []
    depth = 0
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if text.startswith("//", i):
            k = text.find("\n", i)
            i = n if k < 0 else k
            continue
        if text.startswith("/*", i):
            k = text.find("*/", i + 2)
            i = n if k < 0 else k + 2
            continue
        if c == '"':
            i += 1
            while i < n and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
            i += 1
            continue
        if c == "r" and i + 1 < n and text[i + 1] in '"#' and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            j = i + 1
            while j < n and text[j] == "#":
                j += 1
            if j < n and text[j] == '"':
                close = '"' + "#" * (j - i - 1)
                k = text.find(close, j + 1)
                i = n if k < 0 else k + len(close)
                continue
        if c == "'":
            if i + 2 < n and text[i + 1] == "\\":
                k = text.find("'", i + 2)
                i = n if k < 0 else k + 1
                continue
            if i + 2 < n and text[i + 2] == "'":
                i += 3
                continue
        if c in "{([":
            depth += 1
        elif c in "})]":
            depth -= 1
            if depth == 0 and c == "}":
                out.append(i + 1)
        i += 1
    return out


def split_inline(item):
    """Split `...}fn next(...)` where another item starts on the same line as a closing brace."""
    pieces = []
    start = 0
    for offset in closes_at_depth_zero(item):
        rest = item[offset:]
        stripped = rest.lstrip(" ")
        if stripped.startswith("\n") or not stripped:
            continue
        if KEYWORD.match(stripped):
            pieces.append(item[start:offset] + "\n")
            start = offset + (len(rest) - len(stripped))
    pieces.append(item[start:])
    return pieces


def head_line(item):
    for line in item.split("\n"):
        if KEYWORD.match(line):
            return line
    return ""


def item_kind_name(item):
    line = head_line(item)
    match = KEYWORD.match(line)
    if not match:
        return None, None
    kind = match.group(1)
    rest = line[match.end(1):]
    if kind == "impl":
        body = line[match.start(1) + 4:]
        if body.startswith("<"):
            depth = 0
            for index, char in enumerate(body):
                depth += char == "<"
                depth -= char == ">"
                if depth == 0:
                    body = body[index + 1:]
                    break
        header = body.split("{", 1)[0]
        trait, target = (header.split(" for ", 1) if " for " in header else (None, header))
        target_name = IDENT.search(re.sub(r"'[A-Za-z_][A-Za-z0-9_]*", " ", target).replace("&mut ", " ").replace("&", " ").replace("dyn ", " ").replace("mut ", " "))
        trait_name = IDENT.search(trait.replace("unsafe ", "")) if trait else None
        return "impl", (target_name.group(0) if target_name else None, trait_name.group(0) if trait_name else None)
    if kind in ("use", "macro_rules!"):
        return kind, None
    name = IDENT.search(rest)
    return kind, name.group(0) if name else None


def identifiers(text):
    """Identifiers outside comments and string literals."""
    out = set()
    data = text.encode()
    n = len(data)
    i = 0
    while i < n:
        c = data[i]
        if c == 47 and i + 1 < n and data[i + 1] == 47:
            k = data.find(b"\n", i)
            i = n if k < 0 else k
        elif c == 47 and i + 1 < n and data[i + 1] == 42:
            k = data.find(b"*/", i + 2)
            i = n if k < 0 else k + 2
        elif c == 34:
            i += 1
            while i < n and data[i] != 34:
                i += 2 if data[i] == 92 else 1
            i += 1
        elif c == 114 and i + 1 < n and data[i + 1] in (34, 35) and (i == 0 or not (chr(data[i - 1]).isalnum() or data[i - 1] == 95)):
            j = i + 1
            while j < n and data[j] == 35:
                j += 1
            if j < n and data[j] == 34:
                close = b'"' + b"#" * (j - i - 1)
                k = data.find(close, j + 1)
                i = n if k < 0 else k + len(close)
            else:
                i += 1
        elif (65 <= c <= 90) or (97 <= c <= 122) or c == 95:
            j = i
            while j < n and data[j] < 128 and (chr(data[j]).isalnum() or data[j] == 95):
                j += 1
            out.add(data[i:j].decode())
            i = j
        else:
            i += 1
    return out


def split_top_commas(text):
    parts, depth, start, i = [], 0, 0, 0
    while i < len(text):
        c = text[i]
        if text.startswith("->", i):
            i += 2
            continue
        if c in "(<[":
            depth += 1
        elif c in ")>]":
            depth -= 1
        elif c == "," and depth == 0:
            parts.append(text[start:i].strip())
            start = i + 1
        i += 1
    tail = text[start:].strip()
    if tail:
        parts.append(tail)
    return parts


def parse_signature(line, name):
    """`fn NAME<G>(PARAMS) -> RET {` from a head line; returns (generics, params, ret) or None."""
    index = line.find("fn " + name)
    if index < 0:
        return None
    rest = line[index + 3 + len(name):]
    generics = ""
    if rest.startswith("<"):
        depth = 0
        for k, char in enumerate(rest):
            depth += char == "<"
            depth -= char == ">"
            if depth == 0:
                generics, rest = rest[1:k], rest[k + 1:]
                break
    if not rest.startswith("("):
        return None
    depth = 0
    for k, char in enumerate(rest):
        depth += char in "([<"
        if char in ")]" or (char == ">" and rest[k - 1] != "-"):
            depth -= 1
        if depth == 0:
            params, rest = rest[1:k], rest[k + 1:]
            break
    else:
        return None
    rest = rest.strip()
    ret = ""
    if rest.startswith("->"):
        brace = rest.rfind("{") if rest.endswith("{") else rest.find("{ ")
        if brace < 0:
            brace = rest.find("{")
        if brace < 0:
            return None
        ret = rest[2:brace].strip()
        if " where " in ret:
            return None
    elif not rest.startswith("{"):
        return None
    return generics, split_top_commas(params), ret


def declaration_param(param):
    param = param.strip()
    if param.startswith("mut "):
        param = param[4:]
    return param


def param_name(param):
    return declaration_param(param).split(":", 1)[0].strip()


class Group:
    def __init__(self, name):
        self.name = name
        self.texts = []
        self.kind = "pinned"
        self.home = None
        self.refs = set()


def main():
    program_path, plan_path, out = sys.argv[1:4]
    plan = json.load(open(plan_path))
    text = open(program_path, encoding="utf-8").read()
    # Undo codegen-unit wrapping (release and incremental builds).
    text = re.sub(r"(?m)^mod tsrust_generated_unit\d+ \{ use super::\*;\n", "", text)
    text = re.sub(r"(?m)^\}\nuse tsrust_generated_unit\d+::\*;\n", "", text)
    items = [piece for item in split_items(text) for piece in split_inline(item)]
    prologue = []
    groups = {}
    impl_items = []
    for item in items:
        kind, name = item_kind_name(item)
        if kind is None or kind in ("use", "macro_rules!"):
            prologue.append(item)
        elif kind == "impl":
            impl_items.append((item, name))
        else:
            groups.setdefault(name, Group(name)).texts.append(item)
    for item, (target, trait) in impl_items:
        owner = target if target in groups else trait if trait in groups else None
        if owner is None:
            prologue.append(item)
        else:
            groups[owner].texts.append(item)

    def module_name(index):
        return TYPES if index is None else f"tsrust_m{index}"

    # Closure bodies that can move: synchronous inherent `call` methods.
    bodies = {}
    for name, group in groups.items():
        m = re.fullmatch(r"Closure(\d+)", name)
        if not m:
            continue
        for index, item in enumerate(group.texts):
            head = head_line(item)
            if not head.startswith("impl") or " for " in head.split("{", 1)[0]:
                continue
            lines = item.split("\n")
            method = lines[1] if len(lines) > 1 else ""
            sig = parse_signature(method.strip(), "call") if method.strip().startswith("fn call") else None
            if not sig or "&" in sig[2] or "'_" in sig[2] or "impl " in sig[2]:
                continue
            body = Group(name + "#body")
            body.kind = "movable"
            body.home = module_name(plan["closures"][int(m.group(1))])
            body.texts = [item]
            body.closure = name
            body.signature = sig
            body.impl_head = head
            bodies[name] = body
            group.texts.pop(index)
            break
    groups.update({body.name: body for body in bodies.values()})

    for name, group in groups.items():
        m = FUNCTION.fullmatch(name)
        if name == "main":
            group.home = PROGRAM
        elif m:
            group.kind = "movable"
            group.home = module_name(plan["functions"][int(m.group(1))])
        elif re.fullmatch(r"(?:Closure|TsrClosureError)(\d+)", name):
            group.home = module_name(plan["closures"][int(re.search(r"\d+", name).group(0))])
        elif re.fullmatch(r"TsrFunctionError(\d+)", name):
            group.home = module_name(plan["functions"][int(re.search(r"\d+", name).group(0))])
    names = set(groups)
    for group in groups.values():
        found = set()
        for item in group.texts:
            found |= identifiers(item)
        group.refs = {token for token in found if token in names and token != group.name}
        if group.name.endswith("#body"):
            group.refs.add(group.closure)
    signature_refs = {}
    for name, group in groups.items():
        if group.kind == "movable" and not name.endswith("#body"):
            head = head_line(group.texts[0])
            sig = parse_signature(head, name)
            group.signature = sig
            text = "" if sig is None else ",".join(sig[1]) + " -> " + sig[2]
            signature_refs[name] = {token for token in identifiers(text) if token in names}
    users = collections.defaultdict(set)
    for group in groups.values():
        for target in group.refs:
            users[target].add(group.name)
    # Pinned items used by exactly one module stay there; others go to the types crate.
    for name, group in groups.items():
        if group.home is None:
            homes = {groups[user].home for user in users[name] if groups[user].kind == "movable" or groups[user].home == PROGRAM}
            homes.discard(None)
            group.home = homes.pop() if len(homes) == 1 else TYPES
    # A crate sees only itself and the types crate: anything used elsewhere moves up.
    changed = True
    while changed:
        changed = False
        for group in groups.values():
            needs = set(group.refs)
            for target in group.refs:
                if groups[target].kind == "movable" and groups[target].home != group.home:
                    needs |= signature_refs.get(target, set())
            for target in needs:
                other = groups[target]
                if other.kind == "pinned" and other.home not in (group.home, TYPES):
                    other.home = TYPES
                    changed = True
            # A function whose signature cannot be declared joins its callers' crate... not supported: keep in types.
    # Functions without declarable signatures that are called across crates move to the types crate.
    for name, group in groups.items():
        if group.kind == "movable" and not name.endswith("#body") and group.signature is None:
            if any(groups[user].home != group.home for user in users[name]):
                raise SystemExit(f"cannot declare {name}: {head_line(group.texts[0])[:120]}")

    crates = collections.defaultdict(list)
    for name, group in groups.items():
        crates[group.home].append(group)
    externs = collections.defaultdict(dict)  # crate -> {name: declaration}
    exported = set()

    def declaration(name, group):
        generics, params, ret = group.signature
        params = ", ".join(declaration_param(param) for param in params)
        generic = f"<{generics}>" if generics else ""
        arrow = f" -> {ret}" if ret else ""
        return f'#[link_name = "tsrust_{name}"] pub safe fn {name}{generic}({params}){arrow};'

    for group in groups.values():
        for target in group.refs:
            other = groups[target]
            if other.kind == "movable" and not target.endswith("#body") and other.home not in (group.home, TYPES):
                externs[group.home][target] = declaration(target, other)
                exported.add(target)
    # Closure bodies kept apart from their closure: shim in the closure's crate.
    detours = {}
    for name, body in bodies.items():
        closure = groups[name]
        if closure.home == body.home:
            closure.texts.append(body.texts[0])
            crates[body.home].remove(body)
            continue
        detours[name] = body

    def rewrite_paths(text, crate):
        def replace(match):
            target = match.group(1)
            if target not in groups:
                return match.group(0)
            home = groups[target].home
            if home == crate:
                return match.group(0)
            if home == TYPES:
                return f"{TYPES}::{target}"
            if groups[target].kind == "movable":
                return f"crate::tsrust_extern::{target}"
            return match.group(0)
        return re.sub(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*)", replace, text)

    def publish(item):
        """Make a top-level item, its fields and inherent methods public."""
        lines = item.split("\n")
        for index, line in enumerate(lines):
            if not KEYWORD.match(line):
                continue
            attrs = re.match(r"^((?:#\[[^\n]*?\] )*)", line).group(1)
            rest = line[len(attrs):]
            rest = re.sub(r"^pub\((?:crate|super)\) ", "pub ", rest)
            kind = KEYWORD.match(line).group(1)
            if kind in ("impl", "use", "macro_rules!") or rest.startswith("pub ") or rest.startswith("fn main("):
                lines[index] = attrs + rest
            else:
                lines[index] = attrs + "pub " + rest
            head = lines[index]
            body = "\n".join(lines[index:])[len(attrs):]
            prefix = "\n".join(lines[:index]) + ("\n" if index else "") + attrs
            if kind == "struct":
                body = publish_fields(body)
            elif kind == "impl" and " for " not in head.split("{", 1)[0]:
                body = publish_methods(body)
            elif kind == "mod":
                body = re.sub(r"\bpub\((?:crate|super)\) ", "pub ", body)
            return prefix + body
        return item

    def publish_fields(body):
        # First `{`, `(` or `;` outside the generic parameter list.
        angle = 0
        brace = paren = semicolon = -1
        for k, char in enumerate(body):
            if char == "<":
                angle += 1
            elif char == ">" and body[k - 1] != "-":
                angle -= 1
            elif angle == 0 and char in "{(;":
                brace, paren, semicolon = (k if char == "{" else -1), (k if char == "(" else -1), (k if char == ";" else -1)
                break
        if paren >= 0 and (brace < 0 or paren < brace) and (semicolon < 0 or paren < semicolon):
            # Tuple struct: `struct X(A, B);`
            close = paren
            depth = 0
            for k in range(paren, len(body)):
                depth += body[k] in "(<["
                if body[k] in ")]" or (body[k] == ">" and body[k - 1] != "-"):
                    depth -= 1
                if depth == 0:
                    close = k
                    break
            fields = split_top_commas(body[paren + 1:close])
            fields = [field if field.startswith("pub ") else "pub " + field for field in fields]
            return body[:paren + 1] + ", ".join(fields) + body[close:]
        if brace < 0:
            return body
        depth = 0
        for k in range(brace, len(body)):
            depth += body[k] == "{"
            depth -= body[k] == "}"
            if depth == 0:
                return publish_braced(body[:k + 1], brace) + body[k + 1:]
        return body

    def publish_braced(body, brace):
        out = [body[:brace]]
        depth = 0
        start_field = True
        i = brace
        while i < len(body):
            c = body[i]
            if depth == 1 and start_field and (c.isalpha() or c == "_"):
                if not body.startswith("pub", i):
                    out.append("pub ")
                start_field = False
            if c in "{(<[":
                depth += 1
                if depth == 1:
                    start_field = True
            elif c in "})]" or (c == ">" and body[i - 1] != "-"):
                depth -= 1
            elif c == "," and depth == 1:
                start_field = True
            elif c == "#" and depth == 1 and start_field:
                # Field attribute: copy through its closing bracket.
                end = body.find("]", i)
                out.append(body[i:end + 1])
                i = end + 1
                continue
            out.append(c)
            i += 1
        return "".join(out)

    def publish_methods(body):
        out = []
        depth = 0
        i = 0
        while i < len(body):
            c = body[i]
            if c == '"':
                j = i + 1
                while j < len(body) and body[j] != '"':
                    j += 2 if body[j] == "\\" else 1
                out.append(body[i:j + 1])
                i = j + 1
                continue
            if c == "/" and body.startswith("//", i):
                j = body.find("\n", i)
                j = len(body) if j < 0 else j
                out.append(body[i:j])
                i = j
                continue
            keyword = re.match(r"(?:const |unsafe |async )*fn |const [A-Z]|type ", body[i:i + 24]) if depth == 1 and (i == 0 or not (body[i - 1].isalnum() or body[i - 1] == "_")) else None
            if keyword:
                before = "".join(out[-12:])
                if not re.search(r"pub(\([a-z]+\))? $", before):
                    out.append("pub ")
                out.append(keyword.group(0).rstrip("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))
                i += len(keyword.group(0).rstrip("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))
                continue
            if c in "{":
                depth += 1
            elif c in "}":
                depth -= 1
            out.append(c)
            i += 1
        return re.sub(r"\bpub\((?:crate|super)\) ", "pub ", "".join(out))

    os.makedirs(out, exist_ok=True)
    crate_names = sorted(crates, key=lambda c: (c != TYPES, c == PROGRAM, c))
    header = "".join(item for item in prologue if item.lstrip().startswith("#!") or item.lstrip().startswith("//"))
    shared_prologue = "".join(item for item in prologue if not (item.lstrip().startswith("#!") or item.lstrip().startswith("//")))
    manifest = []
    checks = []
    for crate in crate_names:
        parts = [header]
        if crate != TYPES:
            parts.append(f"#[allow(unused_imports)] use {TYPES}::*;\n")
        parts.append(shared_prologue)
        if crate == PROGRAM:
            for other in crate_names:
                if other != PROGRAM:
                    parts.append(f"extern crate {other} as _;\n")
        decls = externs.get(crate, {})
        detour_decls = [(name, body) for name, body in detours.items() if groups[name].home == crate]
        if decls or detour_decls:
            parts.append("pub mod tsrust_extern {\n#[allow(unused_imports)] use super::*;\nunsafe extern \"Rust\" {\n")
            for name in sorted(decls):
                parts.append(decls[name] + "\n")
                checks.append((crate, name, groups[name].home, name))
            for name, body in detour_decls:
                parts.append(detour_declaration(name, body) + "\n")
                checks.append((crate, f"closure_body_{name}", body.home, f"closure_body_{name}"))
            parts.append("}\n}\n#[allow(unused_imports)] use tsrust_extern::*;\n")
        for group in sorted(crates[crate], key=lambda g: g.name):
            for item in group.texts:
                if group.name.endswith("#body") and group.closure in detours:
                    continue
                text = publish(item)
                if group.kind == "movable" and group.name in exported:
                    text = f'#[unsafe(export_name = "tsrust_{group.name}")]\n' + text
                parts.append(rewrite_paths(text, crate))
            if group.name in detours and groups[group.name].home == crate:
                parts.append(detour_shim(group.name, detours[group.name]))
            if group.name.endswith("#body") and group.closure in detours:
                parts.append(rewrite_paths(detour_body(group.closure, group), crate))
        if crate == PROGRAM:
            parts.append("#[allow(dead_code)] fn tsrust_link_checks() {\n")
            for declarer, name, definer, symbol in checks:
                declared = "crate::tsrust_extern" if declarer == PROGRAM else f"{declarer}::tsrust_extern"
                parts.append(f"    let _ = [{declared}::{symbol}, {definer}::{symbol}];\n")
            parts.append("}\n")
        with open(os.path.join(out, f"{crate}.rs"), "w", encoding="utf-8") as handle:
            handle.write("".join(rewrite_paths_prologue(part) for part in parts))
        size = sum(len(part) for part in parts)
        manifest.append({"name": crate, "bytes": size, "dependencies": [] if crate == TYPES else [TYPES] if crate != PROGRAM else [c for c in crate_names if c != PROGRAM]})
    json.dump(manifest, open(os.path.join(out, "crates.json"), "w"), indent=1)
    print(json.dumps({"crates": len(manifest), "types": next(m["bytes"] for m in manifest if m["name"] == TYPES), "externs": sum(len(v) for v in externs.values()), "detours": len(detours), "exported": len(exported)}))


def rewrite_paths_prologue(part):
    return part


def self_type(impl_head, receiver):
    header = impl_head.split("{", 1)[0].strip()
    rest = header[4:]
    generics = ""
    if rest.startswith("<"):
        depth = 0
        for k, char in enumerate(rest):
            depth += char == "<"
            depth -= char == ">"
            if depth == 0:
                generics, rest = rest[1:k], rest[k + 1:]
                break
    target = rest.strip()
    receiver = receiver.strip()
    if receiver in ("self", "mut self"):
        return generics, target, target
    if receiver == "&self":
        return generics, target, f"&{target}"
    if receiver == "&mut self":
        return generics, target, f"&mut {target}"
    raise SystemExit(f"receiver {receiver}")


def detour_parts(name, body):
    generics, params, ret = body.signature
    impl_generics, target, this = self_type(body.impl_head, params[0])
    all_generics = ", ".join(part for part in (impl_generics, generics) if part)
    rest = [declaration_param(param) for param in params[1:]]
    arrow = f" -> {ret}" if ret else ""
    return impl_generics, generics, all_generics, target, this, params, rest, arrow


def detour_declaration(name, body):
    _, _, all_generics, _, this, _, rest, arrow = detour_parts(name, body)
    generic = f"<{all_generics}>" if all_generics else ""
    return f'#[link_name = "tsrust_closure_body_{name}"] pub safe fn closure_body_{name}{generic}(this: {this}{"".join(", " + p for p in rest)}){arrow};'


def detour_shim(name, body):
    impl_generics, generics, _, target, _, params, rest, arrow = detour_parts(name, body)
    generic = f"<{impl_generics}>" if impl_generics else ""
    method_generic = f"<{generics}>" if generics else ""
    args = "".join(", " + param_name(p) for p in params[1:])
    receiver = params[0].strip()
    return f"impl{generic} {target} {{ #[inline(always)] pub fn call{method_generic}({receiver}{''.join(', ' + p for p in rest)}){arrow} {{ tsrust_extern::closure_body_{name}(self{args}) }} }}\n"


def detour_body(name, body):
    impl_generics, generics, all_generics, target, this, params, rest, arrow = detour_parts(name, body)
    lifetimes = impl_generics
    trait_generic = f"<{lifetimes}>" if lifetimes else ""
    trait_args = f"<{lifetimes}>" if lifetimes else ""
    method_generic = f"<{generics}>" if generics else ""
    item = body.texts[0]
    # Original method, renamed, as the implementation of a module-local trait.
    lines = item.split("\n")
    lines[0] = f"impl{trait_generic} TsrBody{name}{trait_args} for {target} {{"
    lines[1] = lines[1].replace("fn call", "fn tsrust_body", 1).replace("pub fn", "fn")
    receiver = params[0].strip()
    trait = f"pub trait TsrBody{name}{trait_generic} {{ fn tsrust_body{method_generic}({receiver}{''.join(', ' + p for p in rest)}){arrow}; }}\n"
    args = "".join(", " + param_name(p) for p in params[1:])
    generic = f"<{all_generics}>" if all_generics else ""
    export = f'#[unsafe(export_name = "tsrust_closure_body_{name}")]\npub fn closure_body_{name}{generic}(this: {this}{"".join(", " + p for p in rest)}){arrow} {{ TsrBody{name}::tsrust_body(this{args}) }}\n'
    return trait + "\n".join(lines) + "\n" + export


if __name__ == "__main__":
    main()
