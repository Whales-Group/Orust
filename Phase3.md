# ORust Phase 3: Modules, imports, show/hide, export

Read `AGENTS.md` first. All hard rules still apply: `rustc` is the source of truth, errors are shown in `.or` terms, and nothing is hidden by default. This phase adds a Dart-style module system that compiles to Rust modules and visibility. Continue numbering from Phase 2 at milestone 15.

## Goal

A developer can split a project across many `.or` files, control what each file exposes, and map the result onto Rust modules and `pub`. Plain `.rs` files in the same project can use exported ORust items, and ORust can import Rust files and crates. Users never write `mod`, `pub`, `pub(crate)`, or `use`.

## Visibility model

Everything is file-private by default. `export` makes something visible to other files.

| ORust | Emitted Rust |
|---|---|
| `class Foo { }` | private `struct Foo` |
| `export class Foo { }` | `pub struct Foo`, public constructor, fields, and non-private methods |
| `export interface Speak { }` | `pub trait Speak` |
| `export enum Shape { }` | `pub enum Shape` |
| `export void helper() { }` | `pub fn helper()` |
| `export const int max = 10;` | `pub const MAX: i32 = 10` |
| `private` member inside an exported class | non-`pub` member |
| `internal` item or member | `pub(crate)` |
| `export shared class Foo { }` | public type/data declarations with the existing shared ownership lowering |

Rules:

- Members cannot be exported from a class that is not exported; report “export the class first”.
- If an exported signature mentions a file-private type, report the friendly E0446/private-interface guidance.
- `show`, `hide`, `as`, `export`, `private`, and `internal` are contextual keywords so existing identifiers with those names remain valid elsewhere.

## Directives

```orust
import 'models/user.or';
import 'models/user.or' show User, Admin;
import 'models/shapes.or' hide Square;
import 'utils/strings.or' as strings;
import 'utils/strings.or' as s show shout;

export 'models/user.or';
export 'models/user.or' show User;
export 'models/shapes.or' hide Square;
```

`export` before a string is a re-export directive; before a declaration it is a visibility modifier.

Imports are non-transitive unless an `export` directive re-exports them. Default imports produce explicit Rust `use` lists, never glob imports. Namespaces rewrite `name.Foo` and `name.helper()` to `name::Foo` and `name::helper()`.

Duplicate imported names are errors unless disambiguated with `show`, `hide`, or `as`. Unused imports warn. Circular imports are allowed and remain subject to Rust's checks. Traits required to call methods on imported interface implementations are imported automatically as `_`.

## Paths and package configuration

Support relative paths (`'user.or'`, `'./user.or'`, `'../shared/x.or'`), package paths (`'package:myapp/models/user.or'`), Rust crates (`'rust:serde_json'`), standard Rust modules, and local Rust files (`'rust:./helpers.rs'`).

```toml
[package]
name = "myapp"
version = "0.1.0"
entry = "main.or"

[dependencies]
otherlib = { path = "../otherlib" }
serde_json = "1"
```

## Module mapping and naming

Each `.or` file becomes a Rust module and directories become parent modules. The generated Cargo tree mirrors the source tree under `target/orust/<package>/src/`. `main.or` becomes `main.rs`; `lib.or` becomes `lib.rs`. `index.or` is a directory barrel; otherwise a `mod.rs` is generated.

File and directory names become snake_case module names, with `r#` added for Rust keywords. ORust camelCase members become snake_case Rust names, types remain PascalCase, and constants become SCREAMING_SNAKE_CASE. `@rustName("exact_name")` overrides the bridge. This stable mapping is documented in `docs/naming.md`.

## Rust interop

`orust build --lib` emits a normal Cargo library crate. Exported ORust items are public to hand-written Rust files. `.rs` files in the source tree are copied unchanged and receive identity span mappings. Rust imports are initially untyped in ORust; users rely on `rustc` for usage checks rather than requiring Rust signature parsing.

## Milestone 17 acceptance shape

```text
models/user.or
models/shapes.or
models/index.or
utils/strings.or
main.or
orust.toml
```

`models/user.or` exports `User` and `Describable`, while keeping `Helper`,
`secret`, and `audit` private. `models/shapes.or` exports `Circle` and
`Square`; `models/index.or` re-exports `User`, `Describable`, and `Circle` but
hides `Square`. `main.or` imports the barrel and aliases the strings module:

```orust
import 'models/index.or';
import 'utils/strings.or' as strings;

void main() {
  var u = new User();
  u.name = "ada";
  print(strings.shout(u.greet()));
  var c = new Circle();
}
```

The generated shape includes `pub mod models`, `pub mod utils`, explicit
`use models::{User, Describable, Circle};`, and `use utils::strings;`. The
negative acceptance cases use `Helper`, `Square`, `u.secret`, and `u.audit()`
from `main.or` and must report the correct source file.

## Diagnostics

The multi-file span table associates every generated span with an `.or` file.
Translated diagnostics include that file path and source line:

| Rust code | Friendly translation |
|---|---|
| E0432 | target file has no export named `Admin`; check spelling or add `export` |
| E0433 | unknown namespace; suggest the missing aliased import |
| E0603 | item is private to its defining file; add `export` there |
| E0616 | field is private to the class; remove `private` or expose a method |
| E0624 | method is private and cannot be called from this file |
| E0252/E0254/E0255 | name collision; suggest `show`, `hide`, or `as` |
| E0412/E0425 | suggest an exact import when another package file exports the name |
| E0446/private_interfaces | public signature mentions a private type; export the type or make the member private |
| unused import warning | identify the unused import path |

“Did you mean?” suggestions use the shortest correct relative path and an
explicit `show Name` clause where appropriate.

## Milestones

15. **Module mapping:** file-to-module generation, directory modules, exported top-level items, basic imports with explicit use lists, `orust.toml`, and multi-file programs.
16. **show / hide / as:** filtering, namespacing, collisions, and unused-import warnings.
17. **Re-exports and barrels:** re-export directives, `index.or`, non-transitive imports, and the acceptance project.
18. **Member visibility:** `private`, `internal`, class-level export fan-out, private-type diagnostics, and automatic trait imports.
19. **Packages and Rust crates:** `package:` imports, ORust/Rust dependencies, and `import 'rust:...'`.
20. **Multi-file diagnostics:** file-aware spans and friendly import/privacy/collision diagnostics.
21. **Rust interop and naming bridge:** `orust build --lib`, mixed trees, naming conversion, `@rustName`, and hand-written Rust integration.
22. **Polish:** `docs/modules.md`, README updates, and locked visibility/naming decisions in `AGENTS.md`.

## Testing

- Golden projects live under `tests/projects/<name>/` with `.or` trees, `orust.toml`, expected emitted Rust, and `expected.err` fixtures.
- Every directive and offline path form becomes a test.
- Add one negative test per diagnostic with the correct `.or` file path.
- Build and run the acceptance project and assert deterministic stdout.
- Test unused `hide` warnings, missing `show` errors, auto trait imports, collisions, and non-transitive imports.
- Keep Rust privacy, type, ownership, and borrow checking delegated to `rustc`.

## Working rules

Follow `AGENTS.md`: do not reimplement Rust privacy or borrow checking, record important choices in `docs/decisions.md`, and emit a clear “not supported yet” note when importing Rust would require parsing full Rust signatures.

Begin with milestone 15.
