---
title: Parse URLs and query parameters
description: Reuse parsed URL owners, inspect borrowed components and replace independent query snapshots.
---

Use `std:url` for URL parsing and form-query parameters. These operations do not
make HTTP requests, resolve DNS or inspect files.

```tsrust
import {parseUrl, resolveUrl, searchParams, setSearchParams} from 'std:url';

function main() {
  const base = unwrap(parseUrl('https://example.com/reports/'));
  const target = unwrap(resolveUrl(base, '../search?tag=one&tag=two'));
  console.log(target.pathname); // /search

  const query = searchParams(target);
  query.append('page', '1');
  setSearchParams(target, query);
  console.log(target.search); // ?tag=one&tag=two&page=1
}
```

Parsing returns `Result<Url, UrlError>`. Keep it as data with `match`, or call
`unwrap` to propagate a typed failure. URL credentials never appear in the
adapter's error text. The serialized URL itself includes credentials when present;
choose fields deliberately when logging.

## Borrow a component, or retain owned text

`href`, `protocol`, `username` and `pathname` return `StringView`. Password,
hostname, search and hash return an optional view. These views borrow the whole
URL, so mutation or transfer waits until their last use. `String(view)` explicitly
copies an independent string. `origin` already returns an owned string; `port`
copies an optional exact integer.

Write components with plain assignments such as `target.pathname = '/new'`.
Rejected writes throw `UrlError`, leaving the URL unchanged. Ports must fit
`0..65535`; undefined removes an optional component. Empty passwords normalize to
absence. Search and hash include their `?`/`#` markers when present; a present
empty query or fragment is distinct from absence.

An owned string RHS follows ordinary assignment ownership: use `clone(text)` when
the caller needs to retain it. Compound assignments and updates on computed URL
properties are unsupported. Readonly URL parameters cannot be assigned through.

## Query snapshots have independent storage

`searchParams(url)` decodes a snapshot. Its mutations never change the URL until
`setSearchParams(url, params)` explicitly replaces the query. Later changes on
either side do not affect the other. An empty replacement list removes the query.

`get` borrows the first value. `getAll` and `entries` copy owned results. Duplicate
keys remain ordered; `set` replaces the first matching value and removes later
duplicates, while `delete` removes every match. Stable `sort` uses Rust string
ordering, including Unicode scalar ordering rather than UTF-16 key ordering.

Form queries decode plus as space and replace invalid decoded UTF-8. For strict
component decoding, use `decodeURIComponent`: it rejects malformed percent
triplets and invalid UTF-8, and leaves plus unchanged. `encodeURIComponent` emits
UTF-8 percent escapes with uppercase hexadecimal digits.

## File URLs follow the host

`pathToFileUrl` requires an absolute native path. It does not infer the working
directory or test whether the path exists. `fileUrlToPath` returns an owned Unicode
native path or an error. Windows drive/UNC rules and Unix root rules remain
platform-specific.

The selected parser normalizes IDNA hosts, IPv6, default ports and dot segments.
It follows its documented Rust backend behavior rather than claiming complete
JavaScript URL compatibility. For example, relative `///test` against an HTTP
base currently returns an empty-host error. Exact API contracts appear in the
[URL reference](/tsrust/reference/library/url/).
