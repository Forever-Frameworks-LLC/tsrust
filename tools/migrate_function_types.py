#!/usr/bin/env python3
"""Rewrite callable ownership and access keywords as built-in types.

    mutate (a: read T) => R      ->  Mutable<Function<R, [a: ReadOnly<T>]>>
    read () => R                 ->  ReadOnly<Function<R>>
    once (..) / owned once (..)  ->  Once<Function<..>>
    owned (..) => R              ->  Function<R, [..]>
    name: consume T              ->  name: Owned<T>

Plain arrow types without a keyword are left as written. Works on raw text so
it also covers sources embedded in Rust string literals; a single-brace
`{name}` format placeholder is treated as one opaque token.
"""
import re
import sys

PREFIX = re.compile(r'\b(owned\s+once|owned|once|mutate|read)(\s*)\(')
ACCESS = re.compile(r'([A-Za-z_0-9}]\??\s*:\s*)(read|mutate|consume)\s+(?=[A-Za-z_({])')
WRAPPER = {'mutate': 'Mutable', 'read': 'ReadOnly', 'once': 'Once', 'owned once': 'Once', 'owned': None, 'consume': 'Owned'}
PLACEHOLDER = re.compile(r'\{[A-Za-z_0-9:?.]*\}')


def close_paren(text, start):
    depth = 0
    for at in range(start, len(text)):
        if text[at] == '(':
            depth += 1
        elif text[at] == ')':
            depth -= 1
            if depth == 0:
                return at
    return None


def type_end(text, start):
    """End of one type that begins at `start` (exclusive index)."""
    depth = 0
    at = start
    while at < len(text):
        ch = text[at]
        if text.startswith('=>', at):
            at += 2
            continue
        if ch == '{':
            match = PLACEHOLDER.match(text, at)
            if match and not text.startswith('{{', at):
                at = match.end()
                continue
            if depth == 0:
                break
            depth += 1
        elif ch in '([<':
            depth += 1
        elif ch in ')]>}':
            if depth == 0:
                break
            depth -= 1
        elif depth == 0 and ch in ',;="\\\n`\'':
            break
        at += 1
    while at > start and text[at - 1] in ' \t':
        at -= 1
    return at


def convert(text):
    position = 0
    while True:
        match = PREFIX.search(text, position)
        if not match:
            break
        opening = match.end() - 1
        closing = close_paren(text, opening)
        arrow = re.match(r'\s*=>\s*', text[closing + 1:]) if closing is not None else None
        if not arrow:
            position = match.end()
            continue
        result_start = closing + 1 + arrow.end()
        result_stop = type_end(text, result_start)
        parameters = text[opening + 1:closing].strip()
        result = text[result_start:result_stop]
        if not result:
            position = match.end()
            continue
        function = f'Function<{result}, [{parameters}]>' if parameters else f'Function<{result}>'
        wrapper = WRAPPER[' '.join(match.group(1).split())]
        replacement = f'{wrapper}<{function}>' if wrapper else function
        text = text[:match.start()] + replacement + text[result_stop:]
        position = match.start()
    position = 0
    while True:
        match = ACCESS.search(text, position)
        if not match:
            break
        start = match.end()
        stop = type_end(text, start)
        if stop <= start:
            position = match.end()
            continue
        replacement = f'{match.group(1)}{WRAPPER[match.group(2)]}<{text[start:stop]}>'
        text = text[:match.start()] + replacement + text[stop:]
        position = match.start() + len(match.group(1))
    return text


def main():
    changed = 0
    for path in sys.argv[1:]:
        with open(path, encoding='utf-8') as handle:
            before = handle.read()
        after = convert(before)
        if after != before:
            with open(path, 'w', encoding='utf-8') as handle:
                handle.write(after)
            changed += 1
    print(f'{changed} of {len(sys.argv) - 1} files changed')


if __name__ == '__main__':
    main()
