/// A simple struct.
pub struct Config {
    pub name: String,
    pub verbose: bool,
}

/// An enum for status.
enum Status {
    Active,
    Inactive,
}

/// A trait for display.
trait Renderable {
    fn render(&self) -> String;
}

/// Top-level function.
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Private helper.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

impl Config {
    pub fn new(name: String) -> Self {
        Config {
            name,
            verbose: false,
        }
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }
}

type Pair = (i32, i32);
