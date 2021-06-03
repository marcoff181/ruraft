use chrono::{Timelike, Utc};
use std::io::prelude::*;
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let listner = TcpListener::bind(("0.0.0.0", 12345))?;
    for stream in listner.incoming() {
        stream.and_then(|mut stream| {
            let ip = stream.peer_addr().unwrap().ip();
            println!("Connection from {}", ip);
            let now = Utc::now();
            let (is_pm, hour) = now.hour12();
            let msg = format!(
                "The current UTC time is {:02}:{:02}:{:02} {}",
                hour,
                now.minute(),
                now.second(),
                if is_pm { "PM" } else { "AM" }
            );
            stream.write_all(msg.as_bytes())?;
            return Ok(());
        })?;
    }

    Ok(())
}
