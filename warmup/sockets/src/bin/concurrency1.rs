use std::sync::mpsc::{channel, Receiver, Sender};
use std::{thread, time};

fn producer(tx: Sender<i32>) {
    for i in 0..10 {
        println!("Producing {}", i);
        tx.send(i).unwrap();
        let wait_time = time::Duration::from_millis(100);
        thread::sleep(wait_time);
    }
    println!("Producer done");
}

fn consumer(rx: Receiver<i32>) {
    for recved in rx {
        println!("Consuming {}", recved);
        let wait_time = time::Duration::from_millis(500);
        thread::sleep(wait_time);
    }
    println!("Consumer goodbye");
}

fn main() -> std::io::Result<()> {
    let (tx, rx) = channel();
    let t1 = thread::spawn(move || {
        producer(tx);
    });
    let t2 = thread::spawn(move || {
        consumer(rx);
    });
    println!("Waiting");
    t1.join().unwrap();
    t2.join().unwrap();
    println!("Goodbye");
    Ok(())
}
