---
title: Embed text files
description: Import UTF-8 text at build time and make an owned copy when needed.
---

An explicit `.txt` import embeds a file's UTF-8 text as a read-only default value.
The application does not execute the file or reopen it at startup.

## Import and read text

After [setting up native execution](/tsrust/tools/rust-setup/), create two files
in the same directory. Put this line in `message.txt`:

```text
Hello from a text file
```

Save this program as `main.tsr`:

```tsrust
import message from "./message.txt";

function main() {
  console.log(message.trim());
  const editableCopy = clone(message);
  console.log(editableCopy.startsWith("Hello"));
}
```

Run `tsrust run main.tsr`. It prints:

```text
Hello from a text file
true
```

The `.txt` extension must be explicit in the import. A text module has one default
export and no named exports. Read the imported string directly; use `clone` when
an owned value is needed. The call to `trim()` above changes the printed result,
not the embedded text.

## Preserve exact text

Imports preserve UTF-8 content, including newlines, a byte-order mark and embedded
null characters. Invalid UTF-8 produces a `module-load` error instead of replacing
invalid bytes. See [strings](/tsrust/language/strings/) for byte offsets and Unicode
scalar positions.

Text files are build inputs. Changing an imported file invalidates the native
cache; rebuilding embeds the new contents. An already-built executable keeps
its embedded version. Inside a [configured project](/tsrust/start/project-structure/), use
`tsrust dev` to rebuild after file changes. The watcher uses the configured build
entry; it does not accept a filename.

## Choose text or structured data

| File | Imported value |
| --- | --- |
| `.txt`, including `.json.txt` | Exact read-only text |
| `.json` | Checked structured data |
| `.jsonc` | Checked structured data with comments and trailing commas allowed |

Importing `.json.txt` does not parse its JSON contents. To decode text at runtime,
use [`JSON.parse<T>`](/tsrust/library/typed-json/). It returns an owned value after
checking the requested shape and throws for malformed JSON or incompatible data.
For build-time data checking, use [JSON/JSONC imports](/tsrust/language/modules/#typed-json-and-jsonc-imports).

## Work in the editor

Imported text offers string completion and hover documentation. Go to definition
opens the text file. Unsaved text edits are available to editor analysis; save the
file before relying on CLI execution or a native rebuild.
