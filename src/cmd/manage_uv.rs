use std::process::Command;
use colored::Colorize;

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
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim() != "y" {
        println!("Exiting...");
        return;
    }
    let output = Command::new("bash")
        .arg("-c")
        .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout.green());
}

fn install_uv_windows() {
    println!("Install Astral UV by running this command:");
    println!("winget install astral-uv.sh");
}
