use crate::meta_data::meta_data_handler::use_node_meta_data;
use crate::meta_data::nodes::node::State;
use crate::utils::config_constants::SHUTTING_DOWN_TIMEOUT_SECS;
use crate::utils::constants::NODES_METADATA_PATH;
use crate::utils::errors::Errors;
use rustls::lock::Mutex;
use std::io::Write;
use std::sync::Arc;
use std::thread::sleep;
use std::{io, thread};
use crate::redistribution::message_sender::MessageSender;

pub struct TerminalInput {
    file: Option<String>,
}

impl Default for TerminalInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalInput {
    pub fn new() -> Self {
        Self { file: None }
    }
    pub fn start_listening(self) {
        let this = Arc::new(Mutex::new(self)); // Wrap `self` in Arc<Mutex>
        thread::spawn(move || {
            loop {
                let data = TerminalInput::get_input();
                let mut instance = this.lock().unwrap(); // Lock the mutex to access `self`
                match instance.match_input(data.as_str()) {
                    Ok(_) => (),
                    Err(e) => println!("{}", e),
                }
            }
        });
    }

    fn get_input() -> String {
        let mut data = String::new();
        io::stdin()
            .read_line(&mut data)
            .expect("Error reading data");
        data.trim().to_string()
    }

    fn match_input(&mut self, input: &str) -> Result<(), Errors> {
        let mut parts = input.splitn(2, ' ');
        let command = parts.next().unwrap_or("");
        let argument = parts.next();

        match command {
            "set_file" => self.set_file(argument),
            "exit" => Self::exit(),
            "pause" => Self::pause(),
            "resume" => Self::resume(),
            "state" => self.state(),
            "states" => Self::states(),
            _ => Err(Errors::Invalid(String::from("Invalid input. Try again."))),
        }
    }

    fn print(&self, data: &str) {
        if let Some(file) = &self.file {
            let mut file = std::fs::File::create(file).expect("Error creating file");
            file.write_all(data.as_bytes())
                .expect("Error writing to file");
        } else {
            println!("{}", data);
        }
    }

    fn set_file(&mut self, argument: Option<&str>) -> Result<(), Errors> {
        if let Some(file_name) = argument {
            self.file = Some(file_name.to_string());
            println!("File is set to: {}", file_name);
            Ok(())
        } else {
            Err(Errors::Invalid(String::from(
                "No file name provided. Usage: set_file <file_name>",
            )))
        }
    }

    fn exit() -> Result<(), Errors> {
        println!("Shutting down...");
        use_node_meta_data(|handler| {
            handler.set_own_node_to_shutting_down(NODES_METADATA_PATH)?;
            handler.update_ranges(NODES_METADATA_PATH)
        })?;
        MessageSender::redistribute()?;
        MessageSender::send_drop_keyspace()?;
        sleep(std::time::Duration::from_secs(
            SHUTTING_DOWN_TIMEOUT_SECS as u64,
        ));
        std::process::exit(0);
    }

    fn pause() -> Result<(), Errors> {
        use_node_meta_data(|handler| {
            if handler
                .get_cluster(NODES_METADATA_PATH)?
                .get_own_node()
                .state
                == State::Active
            {
                handler.set_own_state(NODES_METADATA_PATH, State::StandBy)?
            }
            Ok(())
        })
    }

    fn resume() -> Result<(), Errors> {
        use_node_meta_data(|handler| {
            if handler
                .get_cluster(NODES_METADATA_PATH)?
                .get_own_node()
                .state
                == State::StandBy
            {
                handler.set_own_state(NODES_METADATA_PATH, State::Active)?
            }
            Ok(())
        })
    }

    fn state(&self) -> Result<(), Errors> {
        use_node_meta_data(|handler| {
            let state = handler
                .get_cluster(NODES_METADATA_PATH)?
                .get_own_node()
                .state
                .to_string();
            self.print(&state);
            Ok(())
        })
    }

    fn states() -> Result<(), Errors> {
        Ok(())
    }
}
