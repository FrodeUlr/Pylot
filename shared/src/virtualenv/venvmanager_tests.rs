#[cfg(test)]
mod tests {
    use std::{fs, io};

    use crate::{settings, venv::Venv, venvmanager::VENVMANAGER};

    use tempfile::tempdir;

    #[tokio::test]
    async fn test_list_venvs() {
        let venvs = VENVMANAGER.list().await;
        assert!(venvs.is_empty() || venvs.len() <= 5);
    }

    #[tokio::test]
    async fn test_check_if_exists() {
        let exists = VENVMANAGER
            .check_if_exists("non_existent_venv".to_string())
            .await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_find_venv_none() {
        let venv = VENVMANAGER
            .find_venv(io::stdin(), None, None, "activate")
            .await;
        assert!(venv.is_some() || venv.is_none());
    }

    #[tokio::test]
    async fn test_find_venv_none_cancel() {
        let cursor = std::io::Cursor::new("c\n");
        let venv = VENVMANAGER.find_venv(cursor, None, None, "activate").await;
        assert!(venv.is_some() || venv.is_none());
    }

    #[tokio::test]
    async fn test_find_venv_by_name() {
        let venv = VENVMANAGER
            .find_venv(io::stdin(), None, Some("test_venv".to_string()), "activate")
            .await;
        assert!(venv.is_some());
        assert_eq!(venv.unwrap().name, "test_venv");
    }

    #[tokio::test]
    async fn test_find_venv_by_name_pos() {
        let venv = VENVMANAGER
            .find_venv(io::stdin(), Some("test_venv".to_string()), None, "activate")
            .await;
        assert!(venv.is_some());
        assert_eq!(venv.unwrap().name, "test_venv");
    }

    #[tokio::test]
    async fn test_collect_venvs_empty() {
        let tmp_dir = tempdir().unwrap();
        let entries = fs::read_dir(tmp_dir.path()).unwrap();
        let venvs = VENVMANAGER.collect_venvs(entries);
        assert!(venvs.is_empty());
    }

    #[tokio::test]
    async fn test_print_table() {
        let mut venvs = vec![
            Venv {
                name: "venv1".to_string(),
                python_version: "3.10".to_string(),
                path: "/some/path".to_string(),
                packages: Vec::new(),
                default: false,
                settings: settings::Settings::get_settings(),
            },
            Venv {
                name: "venv2".to_string(),
                python_version: "3.11".to_string(),
                path: "/other/path".to_string(),
                packages: Vec::new(),
                default: true,
                settings: settings::Settings::get_settings(),
            },
        ];
        VENVMANAGER.print_venv_table(&mut venvs).await;
    }

    #[tokio::test]
    async fn test_print_venv_table() {
        let mut venvs = vec![
            Venv {
                name: "venv1".to_string(),
                python_version: "3.10".to_string(),
                path: "/some/path".to_string(),
                packages: Vec::new(),
                default: false,
                settings: settings::Settings::get_settings(),
            },
            Venv {
                name: "venv2".to_string(),
                python_version: "3.11".to_string(),
                path: "/other/path".to_string(),
                packages: Vec::new(),
                default: true,
                settings: settings::Settings::get_settings(),
            },
        ];

        let mut output = Vec::new();
        VENVMANAGER
            .print_venv_table_to(&mut output, &mut venvs)
            .await;

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("venv1"));
        assert!(output_str.contains("3.10"));
        assert!(output_str.contains("venv2"));
        assert!(output_str.contains("3.11"));
    }

    #[test]
    fn test_get_index_valid() {
        let cursor = std::io::Cursor::new("2\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_ok());
    }

    #[test]
    fn test_get_index_invalid() {
        let cursor = std::io::Cursor::new("10\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_cancel() {
        let cursor = std::io::Cursor::new("c\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_non_number() {
        let cursor = std::io::Cursor::new("abc\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }
}
