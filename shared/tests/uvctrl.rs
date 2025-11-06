mod helpers;

#[cfg(test)]
mod tests {
    use shared::uvctrl::{check, install, uninstall, update};

    use crate::helpers::setup_logger;

    #[tokio::test]
    async fn test_check() {
        setup_logger();
        let is_installed = check().await;
        if is_installed {
            println!("Astral UV is installed.");
            assert!(is_installed);
        } else {
            println!("Astral UV is not installed.");
            assert!(!is_installed);
        }
    }

    #[tokio::test]
    async fn test_install_uv_yes() {
        setup_logger();
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor).await.expect("Failed to install Astral UV");
        }
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        setup_logger();
        let cursor = std::io::Cursor::new("n\n");
        install(cursor).await.expect("Failed to install Astral UV");
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        setup_logger();
        let cursor = std::io::Cursor::new("n\n");
        uninstall(cursor)
            .await
            .expect("Failed to uninstall Astral UV");
    }

    #[tokio::test]
    async fn test_update_uv() {
        setup_logger();
        let result = update().await;
        match result {
            Ok(_) => println!("Astral UV updated successfully."),
            Err(e) => println!("Failed to update Astral UV: {}", e),
        }
    }

    #[tokio::test]
    async fn test_install_uv_yes_update() {
        setup_logger();
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor).await.expect("Failed to install Astral UV");
            let result = update().await;
            match result {
                Ok(_) => println!("Astral UV updated successfully."),
                Err(e) => println!("Failed to update Astral UV: {}", e),
            }
        }
    }
}
