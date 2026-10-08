#[derive(Default)]
struct greetArgs {
    name: Option<String>,
    punctuation: Option<String>,
}
fn greet(args: greetArgs) -> i64 {
    let name = args.name.expect("missing required parameter name");
    let punctuation = args.punctuation.unwrap_or("!".to_string());
    return 1;
}
fn main() -> () {
    orust_runtime::install_panic_hook();
    greet(greetArgs { name: Some("Jesse".to_string()), punctuation: None, ..Default::default() });
}
