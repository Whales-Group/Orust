# Drop and cleanup

An ORust class can provide one `on drop` block. It is emitted as Rust's
`Drop::drop`, so cleanup runs automatically when the owner leaves scope and
locals are dropped in reverse declaration order.

```orust
class Guard {
  String name;
  on drop { print(name); }
}

void main() {
  var a = new Guard();
  {
    var b = new Guard();
    print("inner");
  }
  drop a;
  print("end");
}
```

`drop a;` lowers to `drop(a);`. It moves the value and therefore makes `a`
unavailable afterward, just like an explicit Rust move. The `on drop` body is
treated as a mutable receiver context, so a method call such as `close()` is
emitted as `self.close()`.

Drop code cannot await or throw. Cleanup that needs asynchronous work should
be exposed as an explicit `close()` method and awaited by its caller before
the object goes out of scope.
