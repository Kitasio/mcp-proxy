use lib::{AddParams, JsonRpcRequest, JsonRpcResponse};
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
};

struct Server<R, W> {
    reader: BufReader<R>,
    writer: W,
}

impl<R: Read, W: Write> Server<R, W> {
    /// Creates a new Server instance.
    fn new(reader: R, writer: W) -> Self {
        Server {
            reader: BufReader::new(reader),
            writer,
        }
    }

    /// Reads a single JSON-RPC request, processes it, and sends a response.
    /// Returns Ok(false) if EOF is reached, Ok(true) otherwise.
    fn handle_request(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
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

        // Deserialize request
        let request: JsonRpcRequest<AddParams> = serde_json::from_slice(&body)?;

        // Process request (Add method)
        let result = request.params.a + request.params.b;
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        };

        // Serialize and send response
        let response_str = serde_json::to_string(&response)?;
        write!(
            self.writer,
            "Content-Length: {}\r\n\r\n{}",
            response_str.len(),
            response_str
        )?;
        self.writer.flush()?;

        Ok(true) // Signal to continue
    }
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut server = Server::new(stdin.lock(), stdout);

    loop {
        match server.handle_request() {
            Ok(true) => {
                // Request handled, continue loop
            }
            Ok(false) => {
                // EOF detected, break loop
                eprintln!("Client disconnected, shutting down server.");
                break;
            }
            Err(e) => {
                eprintln!("Error handling request: {}", e);
                // For simplicity, break on any error for now.
                break;
            }
        }
    }
}
