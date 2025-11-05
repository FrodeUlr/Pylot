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
