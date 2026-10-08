# `rustplain-render`

Render structured Rust diagnostics as readable terminal output.

The renderer combines `rustplain-diag` records with explanations from
`rustplain-explain`, preserving compiler locations and useful child messages.
It is consumed by the `rustplain` command-line application.
