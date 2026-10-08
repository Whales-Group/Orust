# `rustplain-cargo`

Cargo integration for `rustplain`.

This crate runs Cargo commands with JSON diagnostics enabled, separates build
events from compiler diagnostics, and forwards process status and output to
the `rustplain` rendering layer.
