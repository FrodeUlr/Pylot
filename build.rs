use std::fs;

fn main() {
    println!("cargo:warning=build.rs is running!");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("settings.toml");

    fs::copy("settings.toml", dest_path).expect("Failed to copy settings.toml");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=settings.toml");
}
