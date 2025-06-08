use lib::server::Server;
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut server = Server::new(stdin.lock(), stdout);

    server.run()?;

    Ok(())
}
