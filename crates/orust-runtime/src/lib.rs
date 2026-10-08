pub use async_trait;
pub use futures;
pub use futures::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
pub use tokio;

pub type Stream<T> = Pin<Box<dyn futures::Stream<Item = T> + Send>>;

pub fn runtime_error_code(message: &str) -> &'static str {
    if message.contains("out of range for a List") {
        "OR0010"
    } else if message.contains("already borrowed") {
        "OR0011"
    } else if message.contains("not a char boundary") {
        "OR0012"
    } else if message.contains("attempt to divide by zero") {
        "OR0013"
    } else if message.contains("called `Option::unwrap() on a `None` value")
        || message.contains("called `Option::unwrap()` on a `None` value")
    {
        "OR0014"
    } else if message.contains("overflow") {
        "OR0008"
    } else {
        "OR0099"
    }
}

pub fn friendly_panic_message(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("byte index ") {
        if let Some(index) = rest.split_whitespace().next() {
            return format!("byte {index} is in the middle of a character");
        }
    }
    if message.contains("attempt to divide by zero") {
        return "integer division by zero".into();
    }
    if message.contains("called `Option::unwrap()` on a `None` value") {
        return "tried to unwrap null; check the value before using it".into();
    }
    message.into()
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|panic| {
        let message = panic
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| panic.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("runtime panic");
        let code = runtime_error_code(message);
        let message = friendly_panic_message(message);
        if std::env::var_os("ORUST_BACKTRACE").is_some() {
            eprintln!("ORust runtime error [{code}]: {message}");
            if let Some(location) = panic.location() {
                eprintln!(
                    "at {}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                );
            }
        } else {
            eprintln!("ORust runtime error [{code}]: {message}");
        }
    }));
}

pub fn display_option<T: std::fmt::Display>(value: &Option<T>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

pub struct Channel<T> {
    sender: tokio::sync::mpsc::Sender<T>,
    receiver: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<T>>>,
}

pub fn channel<T: Send + 'static>(capacity: usize) -> Channel<T> {
    let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
    Channel {
        sender,
        receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
    }
}

impl<T: Send + 'static> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: Arc::clone(&self.receiver),
        }
    }
}

impl<T: Send + 'static> Channel<T> {
    pub async fn send(&self, value: T) -> Result<(), Error> {
        self.sender
            .send(value)
            .await
            .map_err(|_| Error::new("channel receiver was dropped"))
    }

    pub async fn recv(&self) -> Option<T> {
        self.receiver.lock().await.recv().await
    }
}
pub use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub struct Error {
    pub message: String,
}
impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self::new(error.to_string())
    }
}

pub struct Task<T>(tokio::task::JoinHandle<T>);
impl<T> Task<T> {
    pub async fn await_task(self) -> Result<T, Error> {
        self.0.await.map_err(|error| Error::new(error.to_string()))
    }
}
pub fn spawn<T: Send + 'static>(
    future: impl std::future::Future<Output = T> + Send + 'static,
) -> Task<T> {
    Task(tokio::spawn(future))
}

pub fn millis(value: u64) -> Duration {
    Duration::from_millis(value)
}
pub fn seconds(value: u64) -> Duration {
    Duration::from_secs(value)
}

#[allow(non_upper_case_globals)]
pub mod std_math {
    pub const pi: f64 = std::f64::consts::PI;
    pub const e: f64 = std::f64::consts::E;

    pub fn sqrt(value: f64) -> f64 {
        value.sqrt()
    }
    pub fn pow(value: f64, exponent: f64) -> f64 {
        value.powf(exponent)
    }
    pub fn sin(value: f64) -> f64 {
        value.sin()
    }
    pub fn cos(value: f64) -> f64 {
        value.cos()
    }
    pub fn tan(value: f64) -> f64 {
        value.tan()
    }
    pub fn log(value: f64) -> f64 {
        value.ln()
    }
    pub fn exp(value: f64) -> f64 {
        value.exp()
    }
    pub fn min(left: f64, right: f64) -> f64 {
        left.min(right)
    }
    pub fn max(left: f64, right: f64) -> f64 {
        left.max(right)
    }
    pub fn abs(value: f64) -> f64 {
        value.abs()
    }
    pub fn floor(value: f64) -> f64 {
        value.floor()
    }
    pub fn ceil(value: f64) -> f64 {
        value.ceil()
    }
}

pub mod std_env {
    pub fn args() -> Vec<String> {
        std::env::args().collect()
    }

    pub fn get(name: String) -> Option<String> {
        std::env::var(name).ok()
    }

    pub fn set(name: String, value: String) {
        std::env::set_var(name, value);
    }

    pub fn cwd() -> String {
        std::env::current_dir()
            .ok()
            .and_then(|path| path.into_os_string().into_string().ok())
            .unwrap_or_default()
    }
}

pub mod std_io {
    use super::Error;

    pub struct File;

    impl File {
        pub fn exists(path: String) -> bool {
            std::path::Path::new(&path).exists()
        }

        pub fn read_string(path: String) -> Result<String, Error> {
            std::fs::read_to_string(path).map_err(|error| Error::new(error.to_string()))
        }

        pub fn write_string(path: String, text: String) -> Result<(), Error> {
            std::fs::write(path, text).map_err(|error| Error::new(error.to_string()))
        }

        pub fn append_string(path: String, text: String) -> Result<(), Error> {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|error| Error::new(error.to_string()))?;
            file.write_all(text.as_bytes())
                .map_err(|error| Error::new(error.to_string()))
        }
    }

    pub struct Path;

    impl Path {
        pub fn join(left: String, right: String) -> String {
            std::path::Path::new(&left)
                .join(right)
                .to_string_lossy()
                .into_owned()
        }

        pub fn parent(path: String) -> Option<String> {
            std::path::Path::new(&path)
                .parent()
                .map(|value| value.to_string_lossy().into_owned())
        }

        pub fn file_name(path: String) -> Option<String> {
            std::path::Path::new(&path)
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
        }
    }
}

pub fn checked_index(index: i64, length: usize) -> usize {
    if index < 0 || index as usize >= length {
        panic!(
            "index {} is out of range for a List of length {}",
            index, length
        );
    }
    index as usize
}

#[cfg(test)]
mod tests {
    use super::{checked_index, friendly_panic_message, runtime_error_code};

    #[test]
    fn accepts_in_range_indices() {
        assert_eq!(checked_index(1, 3), 1);
    }

    #[test]
    #[should_panic(expected = "index -1 is out of range for a List of length 3")]
    fn rejects_negative_indices() {
        checked_index(-1, 3);
    }

    #[test]
    #[should_panic(expected = "index 5 is out of range for a List of length 3")]
    fn rejects_indices_at_or_above_length() {
        checked_index(5, 3);
    }

    #[test]
    fn assigns_stable_runtime_error_codes() {
        assert_eq!(
            runtime_error_code("index -1 is out of range for a List of length 3"),
            "OR0010"
        );
        assert_eq!(
            runtime_error_code("already borrowed: BorrowMutError"),
            "OR0011"
        );
        assert_eq!(runtime_error_code("unexpected panic"), "OR0099");
    }

    #[test]
    fn translates_string_boundary_panics() {
        let raw = "byte index 3 is not a char boundary; it is inside 'é'";
        assert_eq!(runtime_error_code(raw), "OR0012");
        assert_eq!(
            friendly_panic_message(raw),
            "byte 3 is in the middle of a character"
        );
    }

    #[test]
    fn translates_common_numeric_and_option_panics() {
        assert_eq!(runtime_error_code("attempt to divide by zero"), "OR0013");
        assert_eq!(runtime_error_code("attempt to add with overflow"), "OR0008");
        assert_eq!(
            runtime_error_code("called `Option::unwrap()` on a `None` value"),
            "OR0014"
        );
    }
}
