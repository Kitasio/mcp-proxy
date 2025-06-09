use lib::server::Server;
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    // TODO: Define tools and others, and put them into `new`

    let mut server = Server::new(stdin.lock(), stdout);

    server.run()?;

    Ok(())
}
