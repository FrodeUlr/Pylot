#[cfg(test)]
mod tests {
    use pylot::{
        activate, check, create, delete, install, list, print_venvs, uninstall,
        update_packages_from_requirements,
    };
    use shared::{logger, uvvenv};
    use std::io;
    use tokio::fs::{self, write};

    #[tokio::test]
    async fn test_check() {
        logger::initialize_logger(log::LevelFilter::Trace);
        _ = check().await;
    }

    #[tokio::test]
    async fn test_list() {
        logger::initialize_logger(log::LevelFilter::Trace);
        list().await;
    }

    #[tokio::test]
    async fn test_print_venvs_empty() {
        logger::initialize_logger(log::LevelFilter::Trace);
        print_venvs(vec![]).await;
    }

    #[tokio::test]
    async fn test_print_venvs_non_empty() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = uvvenv::UvVenv::new(
            "test_env".to_string(),
            "/path/to/test_env".to_string(),
            "3.8".to_string(),
            vec!["numpy".to_string()],
            false,
        );
        print_venvs(vec![venv]).await;
    }

    #[tokio::test]
    async fn test_delete() {
        logger::initialize_logger(log::LevelFilter::Trace);
        delete(io::stdin(), io::stdin(), Some("test_env".to_string())).await;
    }

    #[tokio::test]
    async fn test_activate() {
        logger::initialize_logger(log::LevelFilter::Trace);
        activate(Some("test_env_not_here".to_string())).await;
    }

    #[tokio::test]
    async fn test_create_missing_name() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let result = create(
            "".to_string(),
            "3.8".to_string(),
            vec![],
            "".to_string(),
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_missing_uv() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("y\n");
        let result_un = uninstall(cursor).await;
        assert!(result_un.is_ok());
        let result = create(
            "test_env".to_string(),
            "3.8".to_string(),
            vec![],
            "".to_string(),
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_packages_from_requirements() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let requirements = "test_requirements.txt".to_string();
        let mut packages = vec!["numpy".to_string()];
        let _ = write(&requirements, "pandas\nscipy\n").await;
        let result = update_packages_from_requirements(requirements.clone(), &mut packages).await;
        assert!(result.is_ok());
        assert!(packages.contains(&"numpy".to_string()));
        assert!(packages.contains(&"pandas".to_string()));
        assert!(packages.contains(&"scipy".to_string()));
        fs::remove_file(requirements).await.unwrap();
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("n\n");
        let result_in = install(cursor.clone()).await;
        assert!(result_in.is_ok());
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("n\n");
        let result_un = uninstall(cursor).await;
        assert!(result_un.is_ok());
    }

    #[tokio::test]
    async fn test_install_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
        }
    }

    #[tokio::test]
    async fn test_install_update_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            pylot::update().await;
            assert!(result_in.is_ok());
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
        }
    }

    #[tokio::test]
    async fn test_uninstall_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
        }
    }

    #[tokio::test]
    async fn test_uninstall_update_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
            pylot::update().await;
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
        }
    }

    #[tokio::test]
    async fn test_create_venv() {
        logger::initialize_logger(log::LevelFilter::Trace);
        #[cfg(unix)]
        {
            use shellexpand::tilde;

            let cursor = std::io::Cursor::new("y\n");
            let cursor_one = std::io::Cursor::new("1\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
            let uv_path = tilde("~/.local/bin/uv");
            std::env::set_var(
                "PATH",
                format!("{}:{}", uv_path, std::env::var("PATH").unwrap()),
            );
            delete(cursor.clone(), io::stdin(), Some("test_env".to_string())).await;
            let result = create(
                "test_env".to_string(),
                "3.11".to_string(),
                vec!["numpy".to_string()],
                "".to_string(),
                true,
            )
            .await;
            log::error!("Result: {:?}", result);
            assert!(result.is_ok());
            let result_exists = create(
                "test_env".to_string(),
                "3.11".to_string(),
                vec!["numpy".to_string()],
                "".to_string(),
                true,
            )
            .await;
            log::error!("Result exists: {:?}", result_exists);
            assert!(result_exists.is_err());
            let result_reqerr = create(
                "test_env2".to_string(),
                "3.11".to_string(),
                vec!["numpy".to_string()],
                "nofiletest".to_string(),
                true,
            )
            .await;
            log::error!("Result reqerr: {:?}", result_reqerr);
            assert!(result_reqerr.is_err());
            let result_pyerr = create(
                "test_env2".to_string(),
                "0.1".to_string(),
                vec!["numpy".to_string()],
                "".to_string(),
                true,
            )
            .await;
            log::error!("Result pyerr: {:?}", result_pyerr);
            assert!(result_pyerr.is_err());
            list().await;
            delete(cursor.clone(), cursor_one, None).await;
            let cursor2 = std::io::Cursor::new("y\n");
            let uv_path = tilde("~/.local/bin/uv");
            std::env::set_var(
                "PATH",
                format!("{}:{}", uv_path, std::env::var("PATH").unwrap()),
            );
            delete(
                cursor2.clone(),
                io::stdin(),
                Some("test_env_def".to_string()),
            )
            .await;
            let result = create(
                "test_env_def".to_string(),
                "3.11".to_string(),
                vec!["pandas".to_string()],
                "".to_string(),
                true,
            )
            .await;
            log::error!("Result: {:?}", result);
            assert!(result.is_ok());
            list().await;
            delete(
                cursor2.clone(),
                io::stdin(),
                Some("test_env_def".to_string()),
            )
            .await;
            let result_un = uninstall(cursor2).await;
            assert!(result_un.is_ok());
        }
    }
}
