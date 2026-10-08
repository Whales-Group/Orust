fn choose(value: Option<String>) -> Option<String> {
    return value.unwrap_or("fallback".to_string());
}
fn main() -> () {
    orust_runtime::install_panic_hook();
    let value = None;
    println!("{}", choose(value));
}
