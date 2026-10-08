use orust_library::ExternalUser;

fn main() {
    println!("hello {}", orust_library::greet("Rust".to_string()));
    println!("label {}", ExternalUser::new().label());
}
