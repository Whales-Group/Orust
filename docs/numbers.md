# ORust numbers

`int` is emitted as `i64`, and `double` as `f64`. Explicit Rust-width types
such as `u8`, `u32`, `usize`, `i128`, and `f32` are preserved. Numeric helper
methods map to Rust's checked, wrapping, and saturating operations:

```orust
void inspect(int value, double decimal) {
  print(value.checkedAdd(1));
  print(value.wrappingAdd(1));
  print(value.saturatingAdd(1));
  print(value.isEven());
  print(value.toDouble());
  print(decimal.round());
}
```

Checked operations return nullable values. Conversions such as `toDouble`,
`toInt`, `round`, `floor`, and `ceil` retain Rust's explicit conversion
semantics in generated code.
