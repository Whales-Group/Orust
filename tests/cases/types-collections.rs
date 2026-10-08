#[derive(Debug)]
struct Values {
    ratio: f64,
    enabled: bool,
    label: String,
    numbers: Vec<i64>,
}
impl Values {
    fn new() -> Self { Self {
        ratio: 1.5,
        enabled: true,
        label: "values".to_string(),
        numbers: vec![1, 2, 3],
    } }
}
fn main() -> () {
    orust_runtime::install_panic_hook();
    let values = Values::new();
    println!("{}", values.label);
    println!("{}", values.numbers);
}
