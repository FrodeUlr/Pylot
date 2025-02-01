use std::process::Stdio;
use colored::Colorize;
use std::io::stdin;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader };

pub async fn install_uv(force: bool) {
    println!("Installing Astral UV, force: {}", force);
    // Check if windows or linux
    if cfg!(target_os = "windows") {
        install_uv_windows().await;
    } else {
        install_uv_linux().await;
    }

}

async fn install_uv_linux() {
    println!("{}", "This will run the following command:".cyan());
    println!("{}", "curl -LsSf https://astral.sh/uv/install.sh | sh".red());
    println!("{}", "Do you want to continue? (y/n): ".cyan());

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    if input.trim() != "y" {
        println!("Exiting...");
        return;
    }

    let mut child = TokioCommand::new("bash")
        .arg("-c")
        .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line.green());
        }
    });

    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line.red());
        }
    });

    let (stdout_res, stderr_res, _) = tokio::join!(stdout_task, stderr_task, child.wait());

    if let Err(e) = stdout_res {
        eprintln!("{}", format!("Error reading stdout: {}", e).red());
    }

    if let Err(e) = stderr_res {
        eprintln!("{}", format!("Error reading stderr: {}", e).red());
    }
}

async fn install_uv_windows() {
    println!("{}", "Install Astral UV by running this command:".cyan());
    println!("{}", "winget install astral-sh.uv".red());
    println!("{}", "Do you want to continue? (y/n): ".cyan());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim() != "y" {
        println!("Exiting...");
        return;
    }
    let mut child = TokioCommand::new("winget")
        .arg("install")
        .arg("astral-sh.uv")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line.green());
        }
    });

    let stderr_taskk = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line.red());
        }
    });

    let (stdout_res, stderr_res, _) = tokio::join!(stdout_task, stderr_taskk, child.wait());

    if let Err(e) = stdout_res {
        eprintln!("{}", format!("Error reading stdout: {}", e).red());
    };
    if let Err(e) = stderr_res {
        eprintln!("{}", format!("Error reading stderr: {}", e).red());
    };

    let _ = child.wait().await;
}
