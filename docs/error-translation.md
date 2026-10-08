# Error translation

The diagnostic translator will consume `cargo check --message-format=json` output after code generation. It will map generated Rust spans back to `.or` spans and select friendly templates by Rust error code. This plumbing begins in milestone 3.

