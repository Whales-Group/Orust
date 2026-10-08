# `orust-lsp`

Language Server Protocol support for ORust `.or` files.

The server provides parsing diagnostics, hover information, completion,
formatting, symbol navigation, document links, and source-aware locations for
ORust projects. It communicates over standard input and output using LSP.

## Installation

Install the binary with the ORust installer or, when published, with:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh | bash
```

The installer asks whether to install `orust-lsp`. To install only the
language server and its runtime dependency, use `--lsp`.

The ORust VS Code extension starts `orust-lsp` from its configured path or
from `PATH`. Rust sections continue to use Rust Analyzer.

See the [main README](../../README.md) and [diagnostics guide](../../docs/diagnostics.md).
