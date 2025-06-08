use lib::{AddParams, JsonRpcRequest, JsonRpcResponse};
use std::io::{self, BufRead, BufReader, Read, Write};

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
    fn handle_request(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        Ok(())
    }
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut server = Server::new(stdin.lock(), stdout);

    loop {
        if let Err(e) = server.handle_request() {
            // In a real server, you might want more sophisticated error handling
            // and potentially send a JSON-RPC error response.
            eprintln!("Error handling request: {}", e);
            // Depending on the error, you might break the loop or continue.
            // For now, we'll just print and continue, assuming transient errors.
        }
    }
}
