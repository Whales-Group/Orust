# `rustplain-diag`

Structured diagnostic types for `rustplain`.

The crate parses Rust and Cargo JSON diagnostic events into serializable
records containing codes, messages, spans, labels, children, and build status.
It is shared by the `rustplain` renderer and Cargo wrapper.
