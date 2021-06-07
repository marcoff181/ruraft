use std::{thread, time};

fn countdown(mut n: i32) {
    while n > 0 {
        println!("T-minus {}", n);
        let one_sec = time::Duration::from_secs(1);
        thread::sleep(one_sec);
        n -= 1;
    }
}

fn countup(stop: i32) {
    let mut x = 0;
    while x < stop {
        println!("Up we go {}", x);
        let one_sec = time::Duration::from_secs(1);
        thread::sleep(one_sec);
        x += 1;
    }
}

fn main() -> std::io::Result<()> {
    let t1 = thread::spawn(|| {
        countdown(10);
    });
    let t2 = thread::spawn(|| {
        countup(5);
    });
    println!("Waiting");
    t1.join().unwrap();
    t2.join().unwrap();
    println!("Goodbye");
    Ok(())
}
