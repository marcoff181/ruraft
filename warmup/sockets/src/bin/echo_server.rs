use std::io::prelude::*;
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let listner = TcpListener::bind(("0.0.0.0", 12345))?;
    for stream in listner.incoming() {
        stream.and_then(|mut stream| {
            let ip = stream.peer_addr().unwrap().ip();
            println!("Connection from {}", ip);

            let mut msg: Vec<u8> = vec![0; 100];
            stream.read(&mut msg)?;
            println!("{}", String::from_utf8_lossy(&msg));
            stream.write_all(&msg)?;
            //stream.flush()?;
            return Ok(());
        })?;
    }

    Ok(())
}
