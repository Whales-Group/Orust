use orust_runtime::*;
async fn fetch() -> i64 {
    orust_runtime::sleep(orust_runtime::seconds(1)).await;
    return 7;
}
#[tokio::main]
async fn main() -> () {
    orust_runtime::install_panic_hook();
    let value = fetch().await;
    println!("{}", value);
}
