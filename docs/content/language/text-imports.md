<!-- tsrust-docs
{"id":"language-text-imports","group":"language","order":35,"status":"partial","targets":["web","native"]}
-->
# Text imports

An explicit `.txt` import embeds the file's exact UTF-8 text as a read-only default
value. It does not execute the file or read a filesystem at application startup.
Newlines, a byte-order mark and embedded null characters are preserved. Invalid
UTF-8 rejects rather than being replaced with guessed characters.

```tsrust
import message from './message.txt';

function main() {
  console.log(message.trim());
  const editableCopy = clone(message);
}
```

Read the imported string directly. Clone it explicitly when an owned value is
needed. Text modules have no named exports. Existing `.json` and `.jsonc` imports
still provide checked structured data; they do not become raw text imports.

## Structured text

`JSON.parse<T>(text)` checks JSON fields, arrays, literal alternatives and numeric
conversions at runtime and returns an owned `T`. Invalid JSON or a value that does
not fit `T` throws a structured JSON error. See the [JSON reference](/standard-library/json).

The documentation generator uses readable `.json.txt` payloads and small typed
loaders. Those files contain JSON text, not thousands of language declarations.
Each page is decoded when loaded. This defers construction; text assets are still
embedded in the executable, not fetched as separate network chunks.
