# Slices and ranges

Range indexing creates a borrowed view:

```orust
var middle = values[1..4];
var prefix = values[..3];
var suffix = values[2..];
var inclusive = values[1..=3];
```

Slice bounds are checked through the ORust runtime helper and are converted to
Rust `usize` indices. A string slice uses byte offsets, matching Rust's
`String`/`str` model; character-aware APIs belong to the string helper surface.
The view remains borrowed, so the normal Rust borrow checker rejects modifying
the source while the slice is live.

Borrowed parameters use Rust slice types: `lend String` becomes `&str`,
`lend List<T>` becomes `&[T]`, and `lend mut List<T>` becomes `&mut [T]` for
slice-compatible access. If the body calls a growth operation such as `add` or
`remove`, it is widened to `&mut Vec<T>` because a slice cannot change length.
