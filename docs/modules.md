# ORust modules

An `.or` file is a Rust module. Directories become parent modules, and the
entry file named by `orust.toml` is the generated crate root. Items are private
unless their declaration starts with `export`; exported classes expose their
constructor, fields, and methods as ordinary Rust `pub` items.

Imports use source paths and are lowered to explicit Rust `use` lists. An
aliased import names the module itself:

```orust
import 'models/user.or';
import 'utils/strings.or' as strings;
import 'models/user.or' show User, Admin hide Secret;
```

The compiler does not add glob imports or a runtime module system. Rust remains
responsible for visibility, types, ownership, and borrow checking.

`show` and `hide` filter the explicit names emitted for an unaliased import;
an alias continues to name the source module. Names not exported by the target
file are rejected by Rust's normal privacy checking rather than being silently
made public.

When an imported declaration has `@rustName`, the generated Rust import keeps
both names available: for example, `ExternalUser as User`. ORust source can
continue to use `User`, while Rust callers can use the declared Rust-facing
name `ExternalUser`.

Re-exporting a module with `export '...' show ...` follows the same bridge and
can be imported by downstream ORust files without losing the Rust-facing name.

Projects may use either `orust.toml` or a standard `Cargo.toml`. The CLI uses
`orust.toml` when both exist; otherwise it detects `Cargo.toml` and defaults to
`src/main.or`. Dependency, feature, dev-dependency, build-dependency, workspace
dependency, target-specific dependency, and Cargo patch/replace entries are
copied into the generated Cargo project, so crates such as `serde_json`,
`reqwest`, and Tokio extensions are resolved by Cargo.
Hand-written `.rs` files are copied into the generated source tree with their
relative paths preserved, so Cargo remains responsible for compiling and
checking them.

Features can be selected through the ORust CLI and are forwarded to Cargo:

```text
orust run src/main.or --features tokio,serde_support
```

Third-party crate APIs can be imported into generated Rust with a top-level
`rust use` declaration and used in a passthrough block:

```orust
rust use serde_json::json;

void main() {
    rust {
        let value = json!({ "source": "Cargo.toml" });
        println!("{}", value["source"]);
    }
}
```

Rust adapter modules remain useful for larger integrations. ORust does not
automatically infer bindings for arbitrary Rust crates; use explicit
`@rustImport`, `@rustType`, qualified Rust types, or `rust` passthrough when
crossing an external API boundary.

Use `orust build --lib file.or` to generate a Cargo library target. Declaration
names may be bridged with `export @rustName("ExactRustName") class User {}`;
the override affects the generated declaration while ORust source names remain
the source-level API.
