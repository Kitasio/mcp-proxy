use lib::{AddParams, JsonRpcRequest, JsonRpcResponse};
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

        Ok(Client { stdin, reader })
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
    // 1. Create a new client instance by spawning the server process.
    let mut client = Client::new("target/debug/hello")?;

    // 2. Prepare the JSON-RPC request.
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "add".to_string(),
        params: AddParams { a: 3, b: 4 },
        id: 1,
    };

    // 3. Send the request using the client.
    client.send_request(&request)?;

    // 4. Read the response using the client.
    let response: JsonRpcResponse<i64> = client.read_response()?;

    // 5. Print the received response.
    println!("Client received: {:?}", response);

    Ok(())
}
