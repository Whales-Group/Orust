#[derive(Debug, Clone)]
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
fn selfRef(&self) -> &Counter {
    return self;
}
}
fn show(value: &Counter) -> () {
    println!("{}", value.get());
}
fn edit(value: &mut Counter) -> () {
    value.inc();
}
fn main() -> () {
    orust_runtime::install_panic_hook();
    let counter = Counter::new();
    edit(&mut counter);
    show(&counter);
    let copyOfCounter = counter.clone();
    println!("{}", copyOfCounter.get());
}
