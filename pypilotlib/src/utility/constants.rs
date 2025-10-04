// Windows-specific constants
pub const PWSH_CMD: &str = "pwsh";
pub const POWERSHELL_CMD: &str = "powershell";

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(PWSH_CMD, "pwsh");
        assert_eq!(POWERSHELL_CMD, "powershell");
    }
}
