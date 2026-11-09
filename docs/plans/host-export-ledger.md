# v0.2 host export ledger

Review of the 1,252 public-API inventory identities that had no review entry
after the 433-identity globals/imports audit. Together the two reviews cover all
1,685 rows of `public_api_inventory` (catalog entries, compiled native bindings,
native type declarations and bundled exports). Reproduce with
`cargo run --offline --locked -p tsrust-compiler --example public_api_inventory`
and `python3 tools/verify-api-audit.py <inventory.json>`.

A row is one spelling, not one independent operation. Counts are not a
conformance percentage.

## Evidence classes

| Class | Rule | Evidence |
|---|---|---|
| family-and-example | Public row with a B01–B19 bundle and a catalog example | Its bundle's `familyWitnessSelectors` cases (positive, rejection, alias, drop and platform witnesses recorded in [stdlib-0.2.0.md](stdlib-0.2.0.md)) plus native execution of every catalog example |
| example | Public row from a pre-v0.2 family (HTTP, schema, SQL, cache, decimal, UUID and receivers) with a catalog example | v0.1 family acceptance retained in its family suite, plus native execution of every catalog example |
| private-helper | `__`-prefixed binding or `*-runtime` module | Compiler lowering target for a public member. It cannot be imported: `check` rejects `import {__setHostname} from 'std:url'` with `module-native`, and `*-runtime` imports with "runtime bindings are private". It is covered through the public member that lowers to it |
| frozen-ui | `std:ui`, `std:ui-runtime` and `Ui*` members | Frozen by the backend-only reset. Its catalog examples run only through the ignored `retained_ui_catalog_examples_match_native_output`. It is not v0.2 backend surface |

Catalog example execution: every non-UI entry carries a fenced example, and
`tools/generate-catalog-suites.py` turns each verified example into a `tsrust test`
case asserting its recorded output. CI runs those suites on all four hosts with
the built compiler; 42 examples that need a database, input file, raw stdout,
their own executable or test registration run in host tests instead.

Host coverage follows the bundle family. Focused platform acceptance is
complete on Linux and ARM macOS at C4/C5 (runs 42480586761 and 42485215015).
Windows and Intel macOS remain open in the B20 native-host checkbox. This ledger
does not claim them.

## Identities by group

| Group | family-and-example | example | private-helper | declaration | frozen-ui |
|---|---:|---:|---:|---:|---:|
| `receiver/type members` | 122 | 292 | 0 | 0 | 3 |
| `std:bytes` | 23 | 0 | 0 | 0 | 0 |
| `std:cache` | 0 | 4 | 15 | 0 | 0 |
| `std:compression` | 16 | 0 | 22 | 0 | 0 |
| `std:crypto` | 17 | 0 | 5 | 0 | 0 |
| `std:date` | 54 | 0 | 0 | 0 | 0 |
| `std:decimal` | 0 | 13 | 0 | 0 | 0 |
| `std:dns` | 1 | 0 | 1 | 0 | 0 |
| `std:env` | 7 | 0 | 0 | 0 | 0 |
| `std:fs` | 29 | 0 | 0 | 0 | 0 |
| `std:futures` | 14 | 0 | 24 | 0 | 0 |
| `std:http` | 53 | 0 | 74 | 0 | 0 |
| `std:io` | 24 | 0 | 10 | 0 | 0 |
| `std:iterator` | 2 | 0 | 0 | 0 | 0 |
| `std:iterator-runtime` | 0 | 0 | 3 | 0 | 0 |
| `std:locale` | 13 | 0 | 4 | 0 | 0 |
| `std:log` | 12 | 0 | 0 | 0 | 0 |
| `std:log-runtime` | 0 | 0 | 9 | 0 | 0 |
| `std:net` | 16 | 0 | 16 | 0 | 0 |
| `std:numeric` | 13 | 0 | 0 | 0 | 0 |
| `std:os` | 3 | 0 | 0 | 0 | 0 |
| `std:path` | 8 | 0 | 0 | 0 | 0 |
| `std:process` | 7 | 0 | 8 | 0 | 0 |
| `std:regex` | 7 | 0 | 11 | 0 | 0 |
| `std:result` | 0 | 1 | 0 | 0 | 0 |
| `std:schema` | 0 | 8 | 26 | 0 | 0 |
| `std:signals` | 4 | 0 | 1 | 0 | 0 |
| `std:sql` | 0 | 6 | 19 | 0 | 0 |
| `std:sqlite` | 0 | 6 | 13 | 0 | 0 |
| `std:testing` | 14 | 0 | 0 | 0 | 0 |
| `std:testing-runtime` | 0 | 0 | 14 | 0 | 0 |
| `std:threads` | 18 | 0 | 16 | 0 | 0 |
| `std:ui` | 0 | 0 | 0 | 0 | 56 |
| `std:ui-runtime` | 0 | 0 | 0 | 0 | 43 |
| `std:url` | 16 | 0 | 27 | 0 | 0 |
| `std:uuid` | 0 | 9 | 0 | 0 | 0 |
| Total | 493 | 339 | 318 | 0 | 102 | 1252

No public row lacks a witness: every non-private, non-UI identity has a catalog
example that runs natively.
