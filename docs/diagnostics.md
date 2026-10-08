# Diagnostics, examples, and exact locations

ORust diagnostics are designed to be actionable in both the terminal and VS
Code. Every diagnostic has a stable code, a source span, and a line/column
location. The editor underline is placed on the smallest useful span; for a
missing token, that span is zero-width immediately where the token belongs.

For example, this source:

```orust
void main() {
  return value
}
```

reports the missing semicolon at the end of `value`, rather than highlighting
the closing brace. The corrected version is:

```orust
void main() {
  return value;
}
```

The same information is available from the CLI:

```text
orust check src/main.or
orust explain OR0005
```

## Common error examples

### OR0001 — moved value

```orust
var first = String("hello");
var second = first;
print(first); // moved value
```

Use `copy first` when a clone is intended, or `lend first` when the callee only
needs a borrow.

### OR0002 — conflicting borrows

```orust
var value = String("hello");
var view = lend value;
edit(value); // cannot mutate while view is active
```

Finish using `view` before mutating `value`, or make the ownership boundary
explicit.

### OR0003 — unavailable import

```orust
import 'models/missing.or';
```

Check the path, the `orust.toml` entry file, and whether the target declaration
is exported.

### OR0005 — invalid lifetime

```orust
lend(source) String echo(lend String source) {
  return source;
}
```

The explicit `lend(source)` relationship tells Rust that the returned borrow
comes from the input. Returning a local value instead would be rejected because
the local is dropped too soon; return an owned value or borrow an input that
outlives the call.

### OR0006 — await outside async

```orust
void main() {
  await load(); // invalid here
}
```

Mark the function `async` and use an async entry point.

### OR0007 — incompatible type or trait

```orust
int count() { return "one"; }
```

Return an `int`, change the declared return type, or convert the value
explicitly.

### OR0016 — refutable binding

```orust
var Some(value) = optional; // may not match
```

Use an `else` branch:

```orust
var Some(value) = optional else { return; };
```

## Documentation diagnostics

Documentation warnings use the same line tracking and are fixable at the
source. For example:

```orust
/// @returns the user
void logUser() {} // OR0609: void functions do not return a value
```

Correct it by removing `@returns` or returning a value. Exported declarations
should include a summary and matching tags:

```orust
/// Loads a user by identifier.
/// @param id the database identifier
/// @returns the loaded user
export User load(int id) { ... }
```

See [error-codes.md](error-codes.md) for the complete registry. In VS Code,
hover a diagnostic for its explanation and example; use **Quick Fix** where a
safe edit is available. Rust errors from generated code are mapped back to the
original `.or` line, while the generated Rust path remains available in the
ORust output channel for debugging.
