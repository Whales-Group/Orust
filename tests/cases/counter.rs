#[derive(Debug)]
struct Counter {
    count: i64,
}
impl Counter {
    fn new() -> Self { Self {
        count: 0,
    } }
fn inc(&mut self) -> () {
    self.count = self.count + 1;
}
fn get(&self) -> i64 {
    return self.count;
}
}
fn show(c: &Counter) -> () {
    println!("{}", c.get());
}
fn main() -> () {
    orust_runtime::install_panic_hook();
    let mut a = Counter::new();
    a.inc();
    show(&a);
    show(&a);
}
