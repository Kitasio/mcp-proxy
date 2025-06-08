use lib::{
    client::Client,
    jsonrpc::{JsonRpcRequest, JsonRpcResponse},
    types::AddParams,
};

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
