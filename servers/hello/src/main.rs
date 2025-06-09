use lib::server::Server;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Define tools and others, and put them into `new`

    let mut server = Server::new();

    server.run()?;

    Ok(())
}
