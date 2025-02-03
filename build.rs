use std::fs;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("settings.toml");

    fs::copy("settings.toml", dest_path).expect("Failed to copy settings.toml");

    println!("cargo:rerun-if-changed=settings.toml");
}
