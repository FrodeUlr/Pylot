#[cfg(test)]
mod tests {
    use pylot::{create, delete, install, list, print_venvs, uninstall};
    use shared::{logger, uvvenv};
    use std::io;

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
            delete(
                cursor.clone(),
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
                cursor.clone(),
                io::stdin(),
                Some("test_env_def".to_string()),
            )
            .await;
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
        }
    }
}
