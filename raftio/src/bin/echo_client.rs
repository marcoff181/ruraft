//#![feature(slice_fill)]

use ::raftio::*;
//use std::io::prelude::*;
use std::net::TcpStream;

fn echo_test(mut sock: TcpStream) -> std::io::Result<()> {
    for n in 0..8 {
        let mut buf: Vec<u8> = vec![0; 10u64.pow(n) as usize];
        for i in 0..10u64.pow(n) as usize {
            buf[i] = b'x';
        }
        //buf.fill(b"x".clone()[0]);

        send_message(&mut sock, buf.clone())?;
        let response = recv_message(&mut sock)?;
        assert_eq!(String::from_utf8(buf).unwrap(), response);
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("localhost:15000")?;
    echo_test(stream)?;
    Ok(())
}
