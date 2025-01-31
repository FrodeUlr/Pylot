use std::process::Command;

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
    println!("Install Astral UV by running this command:");
    let output = Command::new("bash")
        .arg("-c")
        .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("{}", stdout);
    println!("{}", stderr);
}

fn install_uv_windows() {
    println!("Install Astral UV by running this command:");
    println!("winget install astral-uv.sh");
}
