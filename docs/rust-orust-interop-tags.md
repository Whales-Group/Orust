# Rust ↔ ORust interoperability tags

ORust is an object-oriented front end for Rust, not a replacement runtime or a
second standard library. Rust and ORust should be able to live in the same
Cargo package, call each other, share types, and expose APIs in either
direction. Rust remains the authority for ownership, lifetimes, trait
resolution, macros, unsafe code, and final compilation.

This document proposes the explicit tags and contracts for that relationship.
Tags are intentionally opt-in when they affect a public ABI or ownership; the
ordinary ORust syntax should continue to lower to ordinary Rust without tags.

## Existing tags and syntax

| Tag or form | Purpose | Status |
| --- | --- | --- |
| `@rustName("Name")` | Choose the Rust-facing declaration name while retaining the ORust-facing name | Existing |
| `rust { ... }` | Insert Rust source at the generated Rust scope | Existing, textual passthrough |
| `rust use path::Item;` | Emit a crate/module-level Rust `use` declaration | Implemented |
| `lend`, `lend mut`, `copy`, `drop` | Express borrow, mutable borrow, clone, and drop intent | Existing |
| `export` | Make a generated declaration public to Rust | Existing |
| `typedef` | Emit a transparent Rust type alias | Existing |
| `type Name(T)` | Emit a Rust tuple newtype | Existing |
| `implements` / `interface` | Map ORust interfaces to Rust traits and implementations | Existing |

`rust { ... }` is currently an escape hatch. It does not register symbols or
types in the ORust type environment. The tags below turn that escape hatch into
a deliberate, bidirectional interop system.

Even before the typed symbol bridge is added, passthrough code is emitted into
the same generated Rust function scope. Therefore a Rust binding can already be
used by later ORust-generated Rust, provided the name, mutability, and type are
valid to `rustc`:

```orust
void main() {
    rust { let mut value: i64 = 40; }
    value = value + 2;
    print(value);
}
```

This is real shared Rust scope, not a conversion or runtime hand-off. Explicit
type metadata and qualified Rust paths are supported, while the final Rust
type, ownership, and trait checks remain authoritative.

## Declaration tags

### `@rustType("..." )`

Declare the exact Rust type represented by an ORust name.

```orust
@rustType("serde_json::Value")
type JsonValue;
```

This is needed for third-party types such as `serde_json::Value`, `tokio::sync::Mutex`,
and user-defined types from `.rs` files. The declaration must not invent a
parallel ORust representation; it records the Rust type and lets `rustc`
validate its use.

The transparent type-alias form is implemented:

```orust
export @rustType("serde_json::Value") type JsonValue;
```

which emits:

```rust
pub type JsonValue = serde_json::Value;
```

The alias is intentionally transparent: values remain `serde_json::Value` at
the Rust boundary.

### `@rustImport("path")`

Import a Rust declaration into the ORust namespace.

```orust
@rustImport("serde_json::Value")
type JsonValue;

@rustImport("crate::config::load")
JsonValue loadConfig();
```

The generated code should preserve the path or emit the required `use` alias.

Type imports and simple function imports are implemented:

```orust
@rustImport("serde_json::Value")
type JsonValue;

@rustImport("crate::native::load")
String loadValue();
```

The first form emits a Rust `use` alias. The second emits a small Rust wrapper
whose body calls the imported Rust function. Parameters and return values stay
ordinary Rust types, so Cargo and `rustc` validate the actual boundary.

`@externRust("path")` is accepted as the implementation-oriented spelling for
an imported Rust function and uses the same generated wrapper boundary.

### `@orustExport`

Mark a Rust-facing generated declaration as part of the stable ORust API.

```orust
@orustExport
export class Config {}
```

This is useful when a Rust crate consumes generated ORust code and the project
wants to make the boundary explicit for documentation and API checks.

The annotation now promotes the declaration to the Rust-facing public API,
equivalent to `export` for code generation.

### `@repr("...")`

Request a Rust representation for an ORust record, enum, or newtype.

```orust
@repr("C")
export class Point { int x; int y; }
```

Supported values should map directly to Rust attributes such as `C`, `transparent`,
and `u8`. Unsupported values must be rejected rather than silently approximated.

The annotation is now forwarded to the generated declaration.

### `@derive("...")`

Forward selected Rust derives to generated declarations.

```orust
@derive("Debug", "Clone", "Serialize")
export class Config {}
```

The compiler should emit the derive names exactly and let Cargo/rustc resolve
whether the required derive crate is available.

Multiple derive names are accepted and forwarded. Built-in ORust derives are
deduplicated when they overlap with an explicit derive.

### `@externRust`

Declare that the implementation is supplied by Rust rather than generated by
ORust.

```orust
@externRust("crate::native::read_config")
JsonValue readConfig(String path);
```

ORust checks the declared signature as far as possible and emits a call to the
Rust symbol. The Rust compiler remains authoritative for the final signature.

## Expression and scope tags

### `rust let` (implemented)

Declare a Rust value and register its name and type in the ORust environment.

```orust
void main() {
    rust let mut value: serde_json::Value =
        serde_json::json!({ "source": "Cargo.toml" });

    rust value["source"] = serde_json::json!("main.or");
    rust println!("{}", value["source"]);
}
```

The declaration form is implemented as a shared-scope Rust statement. ORust
preserves the explicit Rust type instead of trying to translate
`serde_json::Value` into an ORust built-in type, and later ORust statements can
refer to the binding because both forms lower into the same generated Rust
function scope. Rust remains authoritative for the declaration's type and
mutability.

### `rust expr`

Embed one Rust expression in an ORust expression position.

```orust
JsonValue value = rust serde_json::json!({ "source": "Cargo.toml" });
```

The expression form is implemented. The token sequence is copied into
generated Rust and type-checked by `rustc`; ORust deliberately does not parse
the internals of Rust macros.

### `@rustBlock` (implemented)

Declare that a block intentionally shares the surrounding function scope.

```orust
@rustBlock
rust {
    value["source"] = serde_json::json!("main.or");
}
```

The annotation is accepted on an inline `rust { ... }` block and makes the
shared-scope intent explicit. It does not add an isolation boundary or alter
the Rust tokens.

### `rust use`

Import a Rust path into the generated Rust module.

```orust
rust use tokio::time::sleep;
rust use serde_json::Value as JsonValue;
```

This is supported at the top level of an ORust file. It is emitted in the same
generated module as the surrounding ORust declarations. It affects Rust name
resolution, while ORust expressions may use the imported path through
`rust expr`, `rust let`, or a qualified Rust type annotation.

## Ownership and safety tags

### `@borrowed`

Declare that a Rust-backed value is borrowed and cannot outlive its source.

```orust
@borrowed("'a")
JsonValueView inspect(lend JsonValue value);
```

The generated signature should contain a real Rust lifetime. No runtime bridge
or reference counting should be introduced to hide a lifetime error.

The annotation is implemented as an ownership contract. It requires the
declaration to expose a borrowed input or return, and the generated signature
continues to use Rust references so `rustc` performs the actual lifetime
check.

### `@owned`

Declare that the boundary consumes or returns ownership.

```orust
@owned
JsonValue normalize(JsonValue value);
```

This is primarily documentation and boundary checking; Rust move semantics
remain the source of truth.

The annotation is implemented as boundary metadata and rejects declarations
that also expose borrowed inputs or returns. It does not insert clones or
reference counting.

### `@unsafeRust`

Mark a declaration or block that requires an unsafe Rust context.

```orust
@unsafeRust
rust {
    native_handle.release();
}
```

The compiler should require this marker before emitting an unsafe boundary and
should never make unsafe code implicit.

For an inline block, the marker is implemented as an explicit Rust `unsafe`
scope:

```orust
void release() {
    @unsafeRust rust { native_handle.release(); }
}
```

An unmarked `rust { ... }` block remains unchanged and does not gain unsafe
privileges implicitly.

## Conditional compilation and macros

### `@cfg(...)`

Forward a Rust `cfg` condition to the generated declaration.

```orust
@cfg("feature = \"tokio\"")
async void serve();
```

The condition is forwarded as a Rust `#[cfg(...)]` attribute.

### Rust macros (`rust expr` / `rust { ... }`)

There is no separate `@macro` parser or runtime. Rust macros are intentionally
handled through the existing Rust escape forms, which preserve their token
trees without requiring ORust to understand macro internals:

```orust
var value = rust serde_json::json!({ "source": "Cargo.toml" });
```

Macros remain Rust-owned. ORust only needs to preserve token boundaries and
understand the resulting type when a type annotation is provided. This is the
canonical implementation of the earlier `@macro` design, avoiding a second
macro invocation syntax that could diverge from Rust.

## Compatibility rules

1. Generated ORust code must remain ordinary Rust that can be called from `.rs`
   files.
2. Rust code must be able to call exported ORust functions, methods, traits,
   structs, enums, and type aliases through their generated Rust names.
3. `@rustName` must rename declarations, references, imports, and re-exports
   consistently; source-level ORust names remain stable.
4. Rust types must not be silently wrapped in an ORust runtime object unless a
   tag explicitly requests that representation.
5. Ownership, borrowing, lifetimes, `Send`, `Sync`, trait bounds, and unsafe
   requirements must be checked by `rustc`.
6. Cargo remains the dependency and feature authority. ORust projects may use
   either `Cargo.toml` or `orust.toml`, but generated Rust must enter the normal
   Cargo dependency graph.
7. Every interop tag should have a Rust-only usage example and an ORust-only
   usage example in the test suite.

## Implementation status

1. Shared generated scope and `rust let` symbol registration with explicit Rust types.
2. `rust expr` expression embedding.
4. Consistent `@rustName` rewriting across references and imports.
5. Rust declarations imported through `@rustImport` and `@externRust`.
6. Ownership/lifetime metadata and diagnostics.
7. `@repr`, `@derive`, `@cfg`, and unsafe-boundary validation.
8. Rust macro preservation through the canonical `rust expr` and `rust { ... }`
   boundaries; no separate macro runtime is introduced.

The central design rule is simple: ORust owns the object-oriented syntax, Rust
owns the underlying semantics, and neither language should become second-class
when crossing the boundary.

## Generated-project build invariant

Every executable ORust command must follow the same pipeline:

```text
source/**/*.or
      │
      ▼
filter / translate
      │
      ▼
target/rust/
      │
      ▼
Cargo + rustc
      │
      ▼
target/rust/target/
      │
      ▼
compiled Rust binary and runtime output
```

The generated project is a real Cargo project, not a simulated execution
environment. Dependencies from `Cargo.toml` or `orust.toml`, Rust modules,
features, build scripts, proc macros, and native linking must therefore enter
the normal Cargo graph.

`run` is successful only when Cargo/rustc successfully compiles the generated
Rust and the resulting binary exits successfully. `check` may stop after
compilation, `build` may stop after producing the binary, and `emit` may stop
after printing the generated source, but none of these commands may silently
execute ORust semantics separately from Rust.

`target/rust/` is the single canonical generated project and build workspace.
This avoids duplicate generated trees, follows Cargo conventions, keeps build
artifacts out of source control, and remains inspectable by opening the files
there or using `orust emit`.

## Seamless Rust ↔ ORust flow

The complete workflow should feel like one Cargo build. The user writes
object-oriented code in ORust, writes low-level or ecosystem-specific code in
Rust, and Cargo coordinates the complete dependency graph.

```text
                Cargo.toml / orust.toml
                         │
                         ▼
                 discover package + features
                         │
             ┌───────────┴───────────┐
             ▼                       ▼
         src/*.or                 src/*.rs
             │                       │
             ▼                       │
       ORust filter                  │
       + symbol bridge               │
             │                       │
             ▼                       │
       target/rust/src/*.rs ◄───────┘
             │
             ▼
      Cargo dependency graph
             │
             ▼
          rustc
             │
             ▼
     target/rust/target/
             │
             ▼
       executable / library
```

### Rust consuming ORust

An ORust library is a normal Cargo library after filtering:

```text
orust crate: src/lib.or
        │
        ▼
generated public Rust API
        │
        ▼
Rust crate: use my_orust_crate::Config;
```

When the source project has a `[package] name` in `Cargo.toml`, the generated
library preserves that package identity in `target/rust/Cargo.toml`; a
standalone `.or` file uses `orust-generated` as its fallback package name.

```rust
use my_orust_crate::Config;

fn main() {
    let config = Config::new();
}
```

`export`, `@rustName`, `@repr`, `@derive`, and `@orustExport` control the Rust
facing API. The resulting structs, enums, traits, functions, and methods are
ordinary Rust items; no ORust runtime is required merely to call them.

### ORust consuming Rust

An ORust package declares Rust dependencies in the normal Cargo manifest:

```toml
[dependencies]
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

ORust then imports Rust APIs explicitly:

```orust
@rustImport("serde_json::Value")
type JsonValue;

void main() {
    rust let mut value: JsonValue =
        serde_json::json!({ "source": "Cargo.toml" });

    rust value["source"] = serde_json::json!("main.or");
    rust println!("{}", value["source"]);
}
```

The dependency is resolved by Cargo, the macro is expanded by Rust, and the
value is checked by `rustc`. ORust does not need to know how `serde_json` is
implemented.

### One package, both languages

```text
my-package/
├── Cargo.toml
├── orust.toml
├── src/
│   ├── lib.or
│   ├── service.or
│   ├── native.rs
│   └── lib.rs
└── target/
    └── orust/
        ├── Cargo.toml
        ├── src/
        └── target/
```

The `.or` files provide the object-oriented layer. The `.rs` files provide
direct access to Rust features, unsafe code, proc macros, platform APIs, and
specialized optimizations. Both compile into the same Cargo crate and can call
each other through explicit public Rust-facing declarations.

### Command behavior

```bash
orust check src/main.or   # filter, then cargo check/rustc
orust build src/main.or   # filter, then cargo build
orust run src/main.or     # filter, compile, then run the Rust binary
orust test                 # filter, then cargo test
orust emit src/main.or    # print generated Rust without executing it
```

For `check`, `build`, `run`, and `test`, success always means that the
generated Rust passed Cargo/rustc at the requested stage. There is no separate
ORust execution engine that can disagree with Rust.

The seamless contract is therefore:

1. Write high-level object-oriented code in ORust.
2. Write any native Rust code directly in `.rs` files or `rust` sections.
3. Declare both ecosystems through Cargo.
4. Filter everything into `target/rust/`.
5. Let Cargo and `rustc` resolve, type-check, borrow-check, compile, link, and
   run the complete program.
