use ::raftio::*;
use std::io;
//use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn echo_server(mut sock: TcpStream) -> io::Result<()> {
    loop {
        let msg = recv_message(&mut sock)?;
        send_message(&mut sock, msg)?;
    }
}

fn main() -> std::io::Result<()> {
    let listner = TcpListener::bind(("0.0.0.0", 15000))?;
    for stream in listner.incoming() {
        stream.and_then(|stream| {
            let ip = stream.peer_addr().unwrap().ip();
            println!("Connection from {}", ip);
            let result = echo_server(stream).or_else(|x| {
                println!("Echo Server Error: {:?}", x.kind());
                Err(x)
            });
            println!("{:?}", result);
            return Ok(());
        })?;
    }

    Ok(())
}
