use std::io::{self, prelude::*};

pub fn send_message<W, M>(mut writer: W, msg: M) -> io::Result<()>
where
    W: Write,
    M: Into<Vec<u8>>,
{
    let data: Vec<u8> = msg.into();
    let size = data.len();
    let size = format!("{:10}", size);
    // dbg!(part1.as_bytes().len());
    writer.write_all(size.as_bytes())?;
    writer.write_all(&data)?;
    Ok(())
}

pub fn recv_message<R>(mut reader: R) -> io::Result<String>
where
    R: Read,
{
    let mut size: Vec<u8> = vec![0; 10];
    reader.read_exact(&mut size)?;
    //dbg!(String::from_utf8_lossy(&size));
    let size: usize = String::from_utf8_lossy(&size).trim().parse().unwrap();
    //dbg!(size);
    let mut msg: Vec<u8> = vec![0; size];
    reader.read_exact(&mut msg)?;
    Ok(String::from_utf8_lossy(&msg).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io;
    #[test]
    fn test_send_recv() -> io::Result<()> {
        let wrt = File::create("foo.txt")?;
        send_message(wrt, "hello");

        let rd = File::open("foo.txt")?;
        let msg = recv_message(rd).unwrap();
        assert_eq!(msg, "hello");
        fs::remove_file("foo.txt")?;
        Ok(())
    }
}
