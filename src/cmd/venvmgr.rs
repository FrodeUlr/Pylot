pub struct Venv {
    name: String,
    python_version: String,
    clean: bool,
}

impl Venv {
    pub fn new(name: String, python_version: String, clean: bool) -> Self {
        Venv {
            name,
            python_version,
            clean,
        }
    }

    pub async fn create(&self) {
        println!("Creating virtual environment: {}", self.name);
        println!("Python version: {}", self.python_version);
        println!("Clean: {}", self.clean);
    }
}
