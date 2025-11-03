use std::io::{stdout, BufRead, Write};
use tokio::fs;

pub async fn read_requirements_file(
    requirements: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if !fs::try_exists(requirements).await.unwrap_or(false) {
        return Err("Requirements file does not exist".into());
    }
    let content = tokio::fs::read_to_string(requirements).await?;
    let lines = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    Ok(lines)
}

pub fn confirm<R: std::io::Read>(input: R) -> bool {
    let mut stdin = std::io::BufReader::new(input);
    log::debug!("Do you want to continue? (y/n): ");
    let _ = stdout().flush();
    let mut input_string = String::new();
    if stdin.read_line(&mut input_string).is_ok() {
        matches!(input_string.trim(), "y" | "yes" | "Y" | "YES")
    } else {
        false
    }
}

pub fn which_check(cmd: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let missing_cmds: Vec<&str> = cmd
        .iter()
        .filter(|&&c| which::which(c).is_err())
        .cloned()
        .collect();
    if missing_cmds.is_empty() {
        Ok(())
    } else {
        Err(format!("Missing required commands: {:?}", missing_cmds).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{self, Read};

    struct ErrorReader;

    impl Read for ErrorReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("simulated error".to_string()))
        }
    }

    #[test]
    fn test_confirm_error_returns_false() {
        assert!(!confirm(ErrorReader));
    }

    #[tokio::test]
    async fn test_read_requirements_file() {
        println!("Current dir: {:?}", std::env::current_dir());
        let test_file = "test_requirements.txt";
        let content = "package1\npackage2\n# This is a comment\n\npackage3\n";
        fs::write(test_file, content).await.unwrap();

        println!("Current dir: {:?}", std::env::current_dir());
        let packages = read_requirements_file(test_file)
            .await
            .expect("Failed to read requirements file");
        println!("Current dir: {:?}", std::env::current_dir());
        assert_eq!(packages, vec!["package1", "package2", "package3"]);

        println!("Current dir: {:?}", std::env::current_dir());
        fs::remove_file(test_file).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_requirements_file_nonexistent() {
        let test_file = "nonexistent_requirements.txt";
        let result = std::panic::catch_unwind(|| {
            let _ = tokio::runtime::Handle::current()
                .block_on(async { read_requirements_file(test_file).await });
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_yes() {
        let cursor = std::io::Cursor::new("y\n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_no() {
        let cursor = std::io::Cursor::new("n\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_invalid() {
        let cursor = std::io::Cursor::new("x\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_empty() {
        let cursor = std::io::Cursor::new("\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_whitespace() {
        let cursor = std::io::Cursor::new("   y   \n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_uppercase() {
        let cursor = std::io::Cursor::new("Y\n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_uppercase_yes() {
        let cursor = std::io::Cursor::new("YES\n");
        let result = confirm(cursor);
        assert!(result);
    }
}
