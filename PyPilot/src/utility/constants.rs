// Package: PyPilot
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(NAME, "PyPilot");
        assert_eq!(AUTHORS, "Fulrix");
    }
}
