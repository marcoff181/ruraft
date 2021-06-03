use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("www.python.org:80")?;
    stream.write(b"GET /index.html HTTP/1.0\r\n\r\n");
    // let mut input = String::new();
    // stream.read_to_string(&mut input);
    let mut part: Vec<u8> = vec![0; 1000];
    let mut parts = Vec::new();
    while true {
        let size = stream.read(&mut part)?;
        if size == 0 {
            break;
        }

        parts.extend_from_slice(&part);
    }

    let response = String::from_utf8(parts).unwrap();
    println!("Receive: {}", response);
    Ok(())
}
