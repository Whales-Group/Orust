# ORust strings

ORust `String` values lower to Rust `String` values. The common string
properties and methods map directly to Rust's UTF-8-aware APIs:

```orust
void inspect(String value) {
    print(value.length);             // UTF-8 byte length
    print(value.charCount());        // Unicode scalar count
    print(value.startsWith("OR"));
    print(value.endsWith("st"));
    print(value.replaceAll("old", "new"));
    print(value.toUpperCase());
    print(value.toLowerCase());
    print(value.indexOf("rust"));    // nullable integer result
    print(value.substring(1, 3));
}
```

`length` is the Rust byte length (`String::len`), while `charCount()` counts
Unicode scalar values. `substring` is character-based and uses a half-open
range. String search and replacement use borrowed Rust string-pattern inputs.
