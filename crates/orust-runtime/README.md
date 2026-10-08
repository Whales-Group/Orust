# orust-runtime

Runtime support used by Rust code generated from [ORust](https://github.com/Whales-Group/Orust)
programs.

Most ORust users do not call this crate directly. The `orust` CLI adds it to
generated Cargo projects so async tasks, channels, shared state, and other
ORust runtime features can use Rust's normal ownership and concurrency rules.

The crate is published separately so generated projects can depend on a
versioned crates.io release instead of requiring an ORust source checkout.
