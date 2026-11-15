#!/bin/sh
# Exercise independent lifetimes of the model protocol and diagnostics pipes.
exec 2>&-
IFS= read -r start || exit 1
printf '%s\n' '{"kind":"ready","id":"","text":"","error":""}'
IFS= read -r generate || exit 1
printf '%s\n' '{"kind":"prefix","id":"turn","text":"partial","error":""}'
exit 7
