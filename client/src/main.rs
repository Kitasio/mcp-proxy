use lib::jsonrpc::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use lib::types::{
    AddParams, ClientCapabilities, ClientInfo, ClientRootsCapabilities, InitializeParams,
    InitializeResult,
};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

struct Client {
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
}

impl Client {
    /// Spawns the server process and creates a new Client instance.
    fn new(command: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to take stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to take stdout")?;
        let reader = BufReader::new(stdout);

        let mut client = Client { stdin, reader };

        // Perform Initialization Phase
        client.initialize()?;

        Ok(client)
    }

    /// Performs the JSON-RPC initialization handshake with the server.
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Client: Sending initialize request...");
        let initialize_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "initialize".to_string(),
            params: InitializeParams {
                protocol_version: "2025-03-26".to_string(),
                capabilities: ClientCapabilities {
                    roots: Some(ClientRootsCapabilities {
                        list_changed: Some(true),
                    }),
                    sampling: None,
                    experimental: None,
                },
                client_info: ClientInfo {
                    name: "ExampleClient".to_string(),
                    version: "1.0.0".to_string(),
                },
            },
        };
        self.send_request(&initialize_request)?;

        println!("Client: Reading initialize response...");
        let initialize_response: JsonRpcResponse<InitializeResult> = self.read_response()?;
        println!("Client received: {:?}", initialize_response);

        // Check for initialization errors or version mismatch (basic check for now)
        if let Some(error) = initialize_response.error {
            return Err(format!("Initialization failed: {:?}", error).into());
        }
        if let Some(result) = initialize_response.result {
            if result.protocol_version != "2025-03-26" {
                return Err(format!(
                    "Unsupported server protocol version: {}",
                    result.protocol_version
                )
                .into());
            }
            // TODO: Negotiate capabilities based on result.capabilities
        } else {
            return Err("Initialization response missing result or error".into());
        }

        println!("Client: Sending initialized notification...");
        let initialized_notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/initialized".to_string(),
        };
        self.send_notification(&initialized_notification)?;
        println!("Client: Initialization complete.");

        Ok(())
    }

    /// Sends a JSON-RPC request to the server.
    fn send_request<T: serde::Serialize>(
        &mut self,
        request: &JsonRpcRequest<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request_str = serde_json::to_string(request)?;
        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n{}",
            request_str.len(),
            request_str
        )?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Sends a JSON-RPC notification to the server.
    fn send_notification(
        &mut self,
        notification: &JsonRpcNotification,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let notification_str = serde_json::to_string(notification)?;
        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n{}",
            notification_str.len(),
            notification_str
        )?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Reads a JSON-RPC response from the server.
    fn read_response<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<JsonRpcResponse<T>, Box<dyn std::error::Error>> {
        // Read headers
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            if line == "\r\n" {
                break;
            }
            headers.push_str(&line);
        }

        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .and_then(|s| s.trim().parse::<usize>().ok())
            .ok_or("Missing Content-Length header")?;

        // Read body
        let mut body = vec![0; content_length];
        self.reader.read_exact(&mut body)?;
        let response: JsonRpcResponse<T> = serde_json::from_slice(&body)?;

        Ok(response)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a new client instance by spawning the server process and performing initialization.
    let mut client = Client::new("target/debug/hello")?;

    // 2. Operation Phase (Send the add request)
    println!("Client: Sending add request...");
    let add_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "add".to_string(),
        params: AddParams { a: 3, b: 4 },
        id: 2, // Use a different ID for the next request
    };
    client.send_request(&add_request)?;

    println!("Client: Reading add response...");
    let add_response: JsonRpcResponse<i64> = client.read_response()?;

    // 4. Print the received response.
    println!("Client received: {:?}", add_response);

    Ok(())
}
