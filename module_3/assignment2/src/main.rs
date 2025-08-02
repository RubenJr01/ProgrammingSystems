use std::fs::{File, OpenOptions};
use std::io::{Write, BufReader, BufRead, stdin};
use std::process::Command;

struct LinuxAgent {
    path: String,
}

impl LinuxAgent {
    fn new(path: &str) -> LinuxAgent {
        File::create(path).expect("Unable to create history file");
        LinuxAgent {
            path: path.to_string(),
        }
    }


    fn execute_os_command_linux(&self, command_full: &str) -> String {
        let parts: Vec<&str> = command_full.trim().split_whitespace().collect();
        if parts.is_empty() {
            return "No command entered.".to_string();
        }
        let cmd = parts[0];
        let args = &parts[1..];

        let output = Command::new(cmd)
            .args(args)
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result = format!("{}\n{}", stdout, stderr).trim().to_string();
        result
    }

    fn accept_linux_command(&self) -> String {
        println!("Enter a Linux command (or 'exit' to quit): ");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read input");
        input.trim().to_string()
    }

    fn save_results(&self, content: &str) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)
            .expect("Unable to open history file");
        writeln!(file, "{}", content).expect("Unable to write to file");
    }

    fn show_results(&self) {
        println!("\n--- Command History ---");
        let file = File::open(&self.path).expect("Unable to open history file");
        let reader = BufReader::new(file);
        for (_i, line) in reader.lines().enumerate() {
            println!("{}: {}", _i + 1, line.unwrap())
        }
    }
}

fn main() {
    let agent = LinuxAgent::new("command_history.txt");

    loop {
        let cmd = agent.accept_linux_command();
        if cmd.to_lowercase() == "exit" {
            break;
        }

        let output = agent.execute_os_command_linux(&cmd);
        println!("{}", output);
        let record = format!("$ {}\n{}", cmd, output);
        agent.save_results(&record);
    }

    agent.show_results();
}