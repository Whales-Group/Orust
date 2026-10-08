# `orust-lower`

Semantic lowering support for ORust.

The crate transforms parsed ORust declarations and expressions into the
intermediate structures needed by Rust emission while preserving source
locations and language-level ownership information.

It is used internally by the ORust toolchain. Most users should start with the
[`orust` CLI](../orust-cli/README.md).
