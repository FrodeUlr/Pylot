#[cfg(test)]
mod tests {
    use pylot::{create, delete, install, list};
    use shared::logger;
    use shellexpand::tilde;
    use std::io;
    use tokio::fs::write;

    struct TestContext {
        cursor_yes: std::io::Cursor<&'static str>,
        cursor_one: std::io::Cursor<&'static str>,
        cursor_no: std::io::Cursor<&'static str>,
    }

    impl TestContext {
        async fn setup() -> Self {
            logger::initialize_logger(log::LevelFilter::Trace);
            install(std::io::Cursor::new("y\n")).await.ok();

            let uv_path = tilde("~/.local/bin/uv");
            std::env::set_var(
                "PATH",
                format!("{}:{}", uv_path, std::env::var("PATH").unwrap()),
            );
            TestContext {
                cursor_yes: std::io::Cursor::new("y\n"),
                cursor_one: std::io::Cursor::new("1\n"),
                cursor_no: std::io::Cursor::new("n\n"),
            }
        }
    }

    #[tokio::test]
    async fn test_create_venv_already_exists() {
        #[cfg(unix)]
        {
            let tc = TestContext::setup().await;

            list().await;
            let result = create("test_env", "3.11", vec!["numpy".to_string()], "", true).await;
            log::error!("Result: {:?}", result);
            assert!(result.is_ok());
            let result_exists =
                create("test_env", "3.11", vec!["numpy".to_string()], "", true).await;
            log::error!("Result exists: {:?}", result_exists);
            assert!(result_exists.is_err());
            delete(tc.cursor_no.clone(), tc.cursor_one.clone(), None).await;
            delete(tc.cursor_yes.clone(), tc.cursor_one.clone(), None).await;
        }
    }

    #[tokio::test]
    async fn test_create_venv_invalid_python() {
        #[cfg(unix)]
        {
            TestContext::setup().await;

            list().await;
            let result_pyerr = create("test_env", "0.1", vec!["numpy".to_string()], "", true).await;
            log::error!("Result pyerr: {:?}", result_pyerr);
            assert!(result_pyerr.is_err());
        }
    }

    #[tokio::test]
    async fn test_create_venv_invalid_requirements() {
        #[cfg(unix)]
        {
            TestContext::setup().await;

            list().await;
            let result_reqerr = create(
                "test_env",
                "3.11",
                vec!["numpy".to_string()],
                "nofiletest",
                true,
            )
            .await;
            log::error!("Result reqerr: {:?}", result_reqerr);
            assert!(result_reqerr.is_err());
        }
    }

    #[tokio::test]
    async fn test_create_venv_with_requirements() {
        #[cfg(unix)]
        {
            use tokio::fs;

            let tc = TestContext::setup().await;

            let requirements = "test_requirements.txt";
            let _ = write(&requirements, "pandas\nscipy\n").await;
            list().await;
            let result = create("test_env_req", "3.11", vec![], requirements, true).await;
            log::error!("Result: {:?}", result);
            assert!(result.is_ok());
            list().await;
            delete(tc.cursor_yes.clone(), io::stdin(), Some("test_env_req")).await;
            fs::remove_file(requirements).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_create_venv_defaults() {
        #[cfg(unix)]
        {
            let tc = TestContext::setup().await;

            list().await;
            let result = create("test_env_def", "3.11", vec!["pandas".to_string()], "", true).await;
            log::error!("Result: {:?}", result);
            assert!(result.is_ok());
            list().await;
            delete(tc.cursor_yes.clone(), io::stdin(), Some("test_env_def")).await;
        }
    }
}
