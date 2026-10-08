trait Speak {
    fn speak(&self) -> String;
}
#[derive(Debug)]
struct Dog {
}
impl Dog {
    fn new() -> Self { Self {
    } }
}
impl Speak for Dog {
fn speak(&self) -> String {
    return "woof".to_string();
}
}
fn main() -> () {
    orust_runtime::install_panic_hook();
    let dog = Dog::new();
    println!("{}", dog.speak());
}
