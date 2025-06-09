use lib::client::Client;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new client instance by spawning the server process and performing initialization.
    let mut client = Client::new("target/debug/hello")?;

    println!("Client initialized. Type '/list' to see available tools or type your command.");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    loop {
        // Prompt the user
        print!("> ");
        stdout.flush()?;

        // Read user input
        input.clear();
        stdin.read_line(&mut input)?;
        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        match command {
            "/list" => {
                match client.list_tools(None) {
                    Ok(tools_result) => {
                        println!("Available tools:");
                        for tool in tools_result.tools {
                            println!("  - Name: {}", tool.name);
                            println!("    Description: {}", tool.description);
                            // Optionally print schema: println!("    Schema: {}", tool.input_schema);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error listing tools: {}", e);
                    }
                }
            }
            // Add more commands here later
            _ => {
                println!("Unknown command: {}", command);
                println!("Type '/list' to see available tools.");
            }
        }
    }

    // Note: The client will exit when the user presses Ctrl+C or the server process ends.
}
