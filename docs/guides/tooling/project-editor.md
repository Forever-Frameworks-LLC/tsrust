# Editing a project of several files

The editor checks a file together with the files it imports. This page describes what
you can expect while you edit. It applies to files that use `import` or `export`. A
file without them is checked alone, as before.

## Unsaved files

The text in an open editor tab is the text that is checked. You do not need to save a
dependency to see its effect in the files that import it. When a file is not open, its
text on disk is used.

Diagnostics, hover, completion, signature help, definitions and references follow the
current text of every file in the project. A result that was computed before an edit is
discarded and not shown.

## Help while the code is incomplete

Imported names keep their hover, completion, signature help and definitions while you
type, including when:

- a function body is not finished or not closed;
- a string, template or comment is not finished;
- an `import` statement is still being typed;
- an imported name is not complete or is not exported.

Diagnostics always come from the complete check. Help for incomplete code describes
declarations; it does not mean the project compiles.

```
import { greet } from './greeting.tsr';

function main() {
    greet(   // signature help for greet works here
```

## Completing import names

Between the braces of an `import`, completion lists the exports of the module named
after `from`. Names that the same import already lists are left out. After `as` nothing
is offered, because you write a new local name there.

```
import { gr } from './greeting.tsr';   // completion offers: greet
```

Write the module path first to get this list.

## Quick fixes across files

A quick fix is offered only when the whole project checks after the edit. A fix can
change several files.

- For an open file, the edit carries the version of the editor buffer. The editor
  refuses it if the buffer changed.
- For a file that is not open, the fix is offered only while the file on disk has
  exactly the text that was checked. If the file changes, the fix is withdrawn.

The editor applies an edit to a closed file a short time after it is offered. A change
by another program in that interval is not detected. Open the file first if another
tool writes to it.

## Files that move or change on disk

A dependency that is created, changed, removed or renamed outside the editor
invalidates the results that used it, also before the editor reports the change. A
symbolic link that is removed or pointed at another file is handled the same way.

When one file is open under two addresses (for example through a symbolic link), the
most recent edit supplies the text for checking. Closing one of them keeps the unsaved
text of the other.

## Limits

- Only named imports of functions, types, interfaces and enums: `import { a, b as c }
  from './file.tsr';`.
- A project is the set of files reached from an open file. Files that nothing open
  imports are not checked.
- An unfinished template or block comment hides the rest of its line from import help.
