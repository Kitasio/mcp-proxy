use lib::{AddParams, JsonRpcRequest, JsonRpcResponse};
use std::io::{self, BufRead, Read, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    loop {
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            headers.push_str(&line);
        }

        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .and_then(|s| s.trim().parse::<usize>().ok())
            .expect("Missing Content-Length");

        let mut body = vec![0; content_length];
        reader.read_exact(&mut body).unwrap();

        let request: JsonRpcRequest<AddParams> = serde_json::from_slice(&body).unwrap();

        let result = request.params.a + request.params.b;
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        };

        let response_str = serde_json::to_string(&response).unwrap();
        write!(
            stdout,
            "Content-Length: {}\r\n\r\n{}",
            response_str.len(),
            response_str
        )
        .unwrap();
        stdout.flush().unwrap();
    }
}
