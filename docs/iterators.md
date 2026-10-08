# Iterators and `for`-in

`for (var item in values)` iterates by reference by default. The emitter lowers
this to Rust's `for item in &values`, preserving Rust's borrowing semantics and
avoiding an implicit move of the collection. A consuming form can be added
explicitly through the collection's `consume()` API when ownership of each
element is required.

Iterator adaptor calls remain ordinary Rust iterator calls, so lazy chains such
as `values.where(...).map(...).toList()` keep Rust's lazy behavior until a
terminal operation is requested.

An iterator class can expose the Rust protocol directly:

```orust
class Countdown implements Iterator<int> {
  int? next() { return null; }
}
```

This emits `impl Iterator for Countdown` with `type Item = i64`.

Iterable classes can expose `iterator()` with `implements Iterable<T>`. ORust
then emits `IntoIterator for &Class`, so `for-in` works without moving the
collection. Use `values.consume()` explicitly when the loop should take
ownership.

Supported lazy adaptor names include `map`, `where`, `take`, `skip`, `zip`,
`enumerate`, `expand`, `takeWhile`, `skipWhile`, and `chain`. Terminal names
include `toList`, `toSet`, `toMap`, `fold`, `reduce`, `sum`, `count`, `first`,
`last`, `any`, `every`, `min`, `max`, `join`, and `forEach`.
