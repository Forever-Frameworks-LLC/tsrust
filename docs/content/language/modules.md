<!-- tsrust-docs
{"id":"reference-language-modules","group":"language","order":20,"status":"partial","targets":["web","native"],"examples":["rust-message"]}
-->
# Modules and Rust

Import local application code using ordinary module paths. Public standard libraries use `std:` imports. A local Rust module can expose a checked native binding; target support and transport rules still apply.

The example below imports a real Rust function used by both targets in this documentation app. Optional browser JavaScript integration is a future escape hatch, not an implemented runtime dependency.
