use std::io;

use lib::server::Server;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut server = Server::new(stdin.lock(), stdout);

    loop {
        match server.handle_message() {
            Ok(true) => {
                // Message handled, continue loop
            }
            Ok(false) => {
                // EOF detected, break loop
                eprintln!("Client disconnected, shutting down server.");
                break;
            }
            Err(e) => {
                eprintln!("Error handling message: {}", e);
                // For simplicity, break on any error for now.
                break;
            }
        }
    }
}
