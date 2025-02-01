use std::process::{ Command, Stdio };
use colored::Colorize;
use std::io::{BufRead, BufReader, stdin};

pub fn install_uv(force: bool) {
    println!("Installing Astral UV, force: {}", force);
    // Check if windows or linux
    if cfg!(target_os = "windows") {
        install_uv_windows();
    } else {
        install_uv_linux();
    }

}

fn install_uv_linux() {
    println!("{}", "This will run the following command:".cyan());
    println!("{}", "curl -c -LsSf https://astral.sh/uv/install.sh | sh".red() );
    println!("{}", "Do you want to continue? (y/n): ".cyan());
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    if input.trim() != "y" {
        println!("Exiting...");
        return;
    }
    let mut child = Command::new("bash")
        .arg("-c")
        .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");
 
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line.green()),
                Err(e) => println!("Error: {}", e.to_string().red())
            }
        }
    }

    let _ = child.wait();
}

fn install_uv_windows() {
    println!("{}", "Install Astral UV by running this command:".cyan());
    println!("{}", "winget install astral-sh.uv".red());
    println!("{}", "Do you want to continue? (y/n): ".cyan());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim() != "y" {
        println!("Exiting...");
        return;
    }
    let mut child = Command::new("winget")
        .arg("install")
        .arg("astral-sh.uv")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line.green()),
                Err(e) => println!("Error: {}", e.to_string().red())
            }
        }
    }

    let _ = child.wait();
}
