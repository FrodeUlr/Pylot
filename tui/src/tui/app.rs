use std::collections::HashMap;

pub enum CurrentScreen {
    Main,
    Environment,
    Exiting,
}

pub enum CurrentEnvironment {
    Key,
    Value,
}

pub struct App {
    pub key_input: String,
    pub value_input: String,
    pub pairs: HashMap<String, String>,
    pub current_screen: CurrentScreen,
    pub current_environment: Option<CurrentEnvironment>,
}

//pub trait IApp {
//    fn new() -> Self;
//
//    fn save_key_value(&mut self);
//
//    fn toggle_environment(&mut self);
//
//    fn print_environments(&self);
//}

impl App {
    pub fn new() -> Self {
        Self {
            key_input: String::new(),
            value_input: String::new(),
            pairs: HashMap::new(),
            current_screen: CurrentScreen::Main,
            current_environment: None,
        }
    }

    pub fn save_key_value(&mut self) {
        self.pairs
            .insert(self.key_input.clone(), self.value_input.clone());
        self.key_input = String::new();
        self.value_input = String::new();
        self.current_environment = None;
    }

    pub fn toggle_environment(&mut self) {
        if let Some(environment) = &self.current_environment {
            match environment {
                CurrentEnvironment::Key => {
                    self.current_environment = Some(CurrentEnvironment::Value)
                }
                CurrentEnvironment::Value => {
                    self.current_environment = Some(CurrentEnvironment::Key)
                }
            };
        } else {
            self.current_environment = Some(CurrentEnvironment::Key);
        };
    }

    pub fn print_environments(&self) {
        for (key, value) in &self.pairs {
            println!("{}={}", key, value);
        }
    }
}
