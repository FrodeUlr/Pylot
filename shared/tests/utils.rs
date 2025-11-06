mod helpers;

#[cfg(test)]
mod tests {

    use shared::utils::{confirm, read_requirements_file};
    use tokio::fs;

    use crate::helpers::{setup_logger, ErrorReader};

    #[test]
    fn test_confirm_error_returns_false() {
        assert!(!confirm(ErrorReader));
    }

    #[tokio::test]
    async fn test_read_requirements_file() {
        setup_logger();
        let test_file = "test_requirements.txt";
        let content = "package1\npackage2\n# This is a comment\n\npackage3\n";
        fs::write(test_file, content).await.unwrap();

        let packages = read_requirements_file(test_file)
            .await
            .expect("Failed to read requirements file");
        assert_eq!(packages, vec!["package1", "package2", "package3"]);

        fs::remove_file(test_file).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_requirements_file_nonexistent() {
        setup_logger();
        let test_file = "nonexistent_requirements.txt";
        let result = std::panic::catch_unwind(|| {
            let _ = tokio::runtime::Handle::current()
                .block_on(async { read_requirements_file(test_file).await });
        });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_requirement_file_notexists() {
        setup_logger();
        let result = read_requirements_file("non_existent_file.txt").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_yes() {
        setup_logger();
        let cursor = std::io::Cursor::new("y\n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_no() {
        setup_logger();
        let cursor = std::io::Cursor::new("n\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_invalid() {
        setup_logger();
        let cursor = std::io::Cursor::new("x\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_empty() {
        setup_logger();
        let cursor = std::io::Cursor::new("\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_whitespace() {
        setup_logger();
        let cursor = std::io::Cursor::new("   y   \n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_uppercase() {
        setup_logger();
        let cursor = std::io::Cursor::new("Y\n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_uppercase_yes() {
        setup_logger();
        let cursor = std::io::Cursor::new("YES\n");
        let result = confirm(cursor);
        assert!(result);
    }
}
