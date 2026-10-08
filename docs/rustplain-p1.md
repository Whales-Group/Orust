# rustplain P1

`rustplain` is an offline diagnostic consumer separate from ORust. It accepts
one JSON diagnostic per line from either Cargo's `compiler-message` envelope or
raw rustc JSON and retains structured codes, spans, labels, children,
suggestions, macro expansions, and rendered output.

Try it with a JSON file or stdin:

```sh
cargo run -p rustplain-cli -- --from-json diagnostics.json
cargo check --message-format=json 2>&1 | cargo run -p rustplain-cli -- --from-json -
```

`--raw` prints the input unchanged. P1's fallback explanation is deliberately
conservative; rule-specific ownership explanations, Cargo process wrapping,
and richer renderers are later milestones.
