# ORust diagnostic codes

Friendly diagnostics use stable `OR` codes. The registry starts with the
Phase 4 foundation codes; new codes are appended and existing codes are never
renumbered.

| Code | Rust diagnostics | Meaning |
|---|---|---|
| OR0001 | E0382, E0505 | A value was moved and then used. |
| OR0002 | E0499, E0502 | Borrowing rules prevent simultaneous access. |
| OR0003 | E0432, E0433, E0603 | An import or module path is unavailable. |
| OR0004 | E0616, E0624, E0446 | A private member or type is exposed incorrectly. |
| OR0005 | E0106, E0515, E0597, E0506 | A borrowed value has an invalid lifetime. |
| OR0006 | E0728 | Async-only syntax was used outside an async context. |
| OR0007 | E0277, E0308 | Incompatible types or trait requirements. |
| OR0008 | arithmetic_overflow, overflowing_literals | A numeric calculation or literal overflows. |
| OR0010 | runtime list indexing | An integer index is outside the list bounds. |
| OR0011 | runtime shared borrow | A shared value was borrowed in a conflicting way. |
| OR0012 | string boundary panic | A string slice ended in the middle of a UTF-8 character. |
| OR0013 | divide by zero | Integer division used zero as its divisor. |
| OR0014 | unwrap on null | An optional value was unwrapped while null. |
| OR0015 | trait implementation mismatch | A trait implementation is missing, conflicting, or has an incompatible signature. |
| OR0016 | refutable binding | A destructuring pattern may fail and needs an `else` branch. |
| OR0017 | unreachable pattern | A match pattern is shadowed by an earlier pattern. |
| OR0099 | runtime panic | The runtime received an otherwise-unclassified panic. |

The original Rust code remains available in debug output, but user-facing
messages use these stable ORust codes and `.or` source locations.
