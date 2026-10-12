# Borrowed JSON Lines reader API audit

JsonlReaderView and borrowJsonl extend std:json without adding globals. The name
states the exclusive borrow of an existing BufferedReader. No file is reopened,
no capacity is silently selected, and no second read-ahead buffer is allocated.
Limits match openJsonl and validate before input consumption. Error positions are
relative to adapter creation because the source may already have been read.

Existing readJsonl, readJsonl<T> and closeJsonl accept the view. Typed output still
owns independent storage. The adapter has a real source lifetime, cannot be
cloned or forged, and excludes owner operations while live. Closing the adapter
releases its borrow without closing the file; the caller retains explicit close
responsibility. No hidden shared owner or automatic source copy is introduced.
The inventory grows from 427 to 429 identities. Parameter help and executable
examples describe these boundaries, including relative error positions.
