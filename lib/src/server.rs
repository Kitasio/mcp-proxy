use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub logging: Option<ServerLoggingCapabilities>,
    pub prompts: Option<ServerPromptsCapabilities>,
    pub resources: Option<ServerResourcesCapabilities>,
    pub tools: Option<ServerToolsCapabilities>,
    pub experimental: Option<serde_json::Value>, // Use Value for flexibility
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerLoggingCapabilities {} // Placeholder

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPromptsCapabilities {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerResourcesCapabilities {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerToolsCapabilities {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
    pub instructions: Option<String>,
}

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
};

use crate::{
    client::InitializeParams,
    jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse},
    types::AddParams,
};

#[derive(Copy, Clone)]
pub enum ServerState {
    Uninitialized,
    Initializing, // Sent initialize response, waiting for initialized notification
    Initialized,
}

pub struct Server<R, W> {
    pub reader: BufReader<R>,
    pub writer: W,
    pub state: ServerState,
}

impl<R: Read, W: Write> Server<R, W> {
    /// Creates a new Server instance.
    pub fn new(reader: R, writer: W) -> Self {
        Server {
            reader: BufReader::new(reader),
            writer,
            state: ServerState::Uninitialized,
        }
    }

    /// Reads a single JSON-RPC message, processes it, and sends a response if applicable.
    /// Returns Ok(false) if EOF is reached, Ok(true) otherwise.
    pub fn handle_message(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // Read headers
        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            let bytes_read = self.reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Ok(false); // Client disconnected
            }

            let line = line.trim();
            if line.is_empty() {
                break; // End of headers
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        // Safely extract Content-Length
        let content_length = headers
            .get("content-length")
            .ok_or("Missing Content-Length header")?
            .parse::<usize>()
            .map_err(|_| "Invalid Content-Length")?;

        // Read body
        let mut body = vec![0; content_length];
        self.reader.read_exact(&mut body)?;

        // Attempt to deserialize as a generic JSON-RPC message to get method and id
        let raw_message: serde_json::Value = serde_json::from_slice(&body)?;
        let method = raw_message["method"].as_str().ok_or("Missing method")?;
        let id = raw_message["id"].as_u64(); // id is optional for notifications

        match (self.state, method) {
            (ServerState::Uninitialized, "initialize") => {
                eprintln!("Server: Received initialize request");
                let request: JsonRpcRequest<InitializeParams> =
                    serde_json::from_value(raw_message)?;

                // Basic version negotiation: only support the specified version
                if request.params.protocol_version != "2025-03-26" {
                    let response = JsonRpcResponse::<serde_json::Value> {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602, // Invalid params
                            message: "Unsupported protocol version".to_string(),
                            // data: Some(serde_json::json!({ "supported": ["2025-03-26"], "requested": request.params.protocol_version })), // Optional data
                        }),
                        id: request.id,
                    };
                    self.send_response(&response)?;
                    // Protocol error, maybe shut down? For now, just return true.
                    return Ok(true);
                }

                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(InitializeResult {
                        protocol_version: "2025-03-26".to_string(),
                        capabilities: ServerCapabilities {
                            logging: Some(ServerLoggingCapabilities {}),
                            prompts: Some(ServerPromptsCapabilities {
                                list_changed: Some(true),
                            }),
                            resources: Some(ServerResourcesCapabilities {
                                subscribe: Some(true),
                                list_changed: Some(true),
                            }),
                            tools: Some(ServerToolsCapabilities {
                                list_changed: Some(true),
                            }),
                            experimental: None,
                        },
                        server_info: ServerInfo {
                            name: "ExampleServer".to_string(),
                            version: "1.0.0".to_string(),
                        },
                        instructions: Some(
                            "Welcome! Send 'add' requests after initialization.".to_string(),
                        ),
                    }),
                    error: None,
                    id: request.id,
                };
                self.send_response(&response)?;
                self.state = ServerState::Initializing; // Move to next state
                Ok(true)
            }
            (ServerState::Initializing, "notifications/initialized") => {
                eprintln!("Server: Received initialized notification");
                // No params to deserialize for this notification
                self.state = ServerState::Initialized; // Move to next state
                Ok(true)
            }
            (ServerState::Initialized, "add") => {
                eprintln!("Server: Received add request");
                let request: JsonRpcRequest<AddParams> = serde_json::from_value(raw_message)?;

                let result = request.params.a + request.params.b;
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(result),
                    error: None,
                    id: request.id,
                };
                self.send_response(&response)?;
                Ok(true)
            }
            (ServerState::Uninitialized, _) => {
                // Received a request other than initialize before initialization
                if let Some(id) = id {
                    let response = JsonRpcResponse::<serde_json::Value> {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32002, // Server not initialized
                            message:
                                "Server not initialized. 'initialize' must be the first request."
                                    .to_string(),
                        }),
                        id,
                    };
                    self.send_response(&response)?;
                } else {
                    // Received a notification before initialization, just ignore? Or log error?
                    eprintln!(
                        "Server: Received notification '{}' before initialization. Ignoring.",
                        method
                    );
                }
                Ok(true) // Continue processing
            }
            (ServerState::Initializing, _) => {
                // Received any other message while in the Initializing state
                if let Some(id) = id {
                    let response = JsonRpcResponse::<serde_json::Value> {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32002, // Server not initialized
                            message: format!(
                                "Server is initializing. Received unexpected method '{}'. Waiting for 'notifications/initialized'.",
                                method
                            ),
                        }),
                        id,
                    };
                    self.send_response(&response)?;
                } else {
                    eprintln!(
                        "Server: Received unexpected notification '{}' while initializing. Ignoring.",
                        method
                    );
                }
                Ok(true) // Continue processing
            }
            (ServerState::Initialized, method) => {
                // Received an unknown method after initialization
                if let Some(id) = id {
                    let response = JsonRpcResponse::<serde_json::Value> {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601, // Method not found
                            message: format!("Method not found: '{}'", method),
                        }),
                        id,
                    };
                    self.send_response(&response)?;
                } else {
                    eprintln!(
                        "Server: Received unknown notification '{}'. Ignoring.",
                        method
                    );
                }
                Ok(true) // Continue processing
            }
        }
    }

    /// Sends a JSON-RPC response to the client.
    pub fn send_response<T: serde::Serialize>(
        &mut self,
        response: &JsonRpcResponse<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response_str = serde_json::to_string(response)?;
        write!(
            self.writer,
            "Content-Length: {}\r\n\r\n{}",
            response_str.len(),
            response_str
        )?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("Server: Starting message loop...");
        loop {
            match self.handle_message() {
                Ok(true) => {
                    // Message handled, continue loop
                }
                Ok(false) => {
                    // EOF detected, break loop
                    eprintln!("Server: Client disconnected, shutting down.");
                    return Ok(()); // Clean shutdown
                }
                Err(e) => {
                    eprintln!("Server: Error handling message: {}", e);
                    // Decide how to handle errors: return the error, or continue?
                    // For now, let's return the error to stop the server.
                    return Err(e);
                }
            }
        }
    }
}
