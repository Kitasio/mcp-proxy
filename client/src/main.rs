use lib::client::Client;
use serde_json::json;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new client instance by spawning the server process and performing initialization.
    let mut client = Client::new("target/debug/hello")?;

    println!("Client initialized. Available commands:");
    println!("  /list - List available tools");
    println!("  /greet <name> - Call the greet tool with a name");
    println!("  /time - Call the get_time tool");
    println!("  /quit - Exit the client");

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

        let parts: Vec<&str> = command.split_whitespace().collect();
        match parts.get(0) {
            Some(&"/list") => {
                match client.list_tools(None) {
                    Ok(tools_result) => {
                        println!("Available tools:");
                        for tool in tools_result.tools {
                            println!("  - Name: {}", tool.name);
                            println!("    Description: {}", tool.description);
                            println!("    Schema: {}", tool.input_schema);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error listing tools: {}", e);
                    }
                }
            }
            Some(&"/greet") => {
                if let Some(name) = parts.get(1) {
                    let arguments = json!({ "name": name });
                    match client.call_tool("greet".to_string(), arguments) {
                        Ok(result) => {
                            println!("Tool result (error: {}):", result.is_error);
                            for content in result.content {
                                if content.content_type == "text" {
                                    if let Some(text) = content.content.get("text").and_then(|v| v.as_str()) {
                                        println!("  {}", text);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error calling greet tool: {}", e);
                        }
                    }
                } else {
                    println!("Usage: /greet <name>");
                }
            }
            Some(&"/time") => {
                let arguments = json!({});
                match client.call_tool("get_time".to_string(), arguments) {
                    Ok(result) => {
                        println!("Tool result (error: {}):", result.is_error);
                        for content in result.content {
                            if content.content_type == "text" {
                                if let Some(text) = content.content.get("text").and_then(|v| v.as_str()) {
                                    println!("  {}", text);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error calling get_time tool: {}", e);
                    }
                }
            }
            Some(&"/quit") => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command: {}", command);
                println!("Available commands: /list, /greet <name>, /time, /quit");
            }
        }
    }

    Ok(())
}
