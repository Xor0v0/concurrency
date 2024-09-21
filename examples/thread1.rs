use anyhow::{Ok, Result};
use std::{
    sync::mpsc::{self, channel},
    thread,
    time::Duration,
};
const NUM_PRODUCERS: usize = 4;

struct Msg {
    idx: usize,
    value: i32,
}

fn producer(tx: mpsc::Sender<Msg>, idx: usize) -> Result<()> {
    loop {
        let value = rand::random::<i32>();
        tx.send(Msg::new(idx, value)).unwrap();
        let sleep_time = rand::random::<u8>() as u64 * 10;
        thread::sleep(Duration::from_millis(sleep_time));
        if value % 5 == 0 {
            break;
        }
    }
    println!("Producer {} exit!", idx);
    Ok(())
}

impl Msg {
    fn new(idx: usize, value: i32) -> Self {
        Msg { idx, value }
    }
}

fn main() -> Result<()> {
    let (tx, rx) = channel();
    for idx in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(tx, idx));
    }
    drop(tx);

    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("Consumer: idx={}, value={}", msg.idx, msg.value);
        }
        println!("Consumer exit!");
        114514
    });

    let secret = consumer
        .join()
        .map_err(|e| anyhow::anyhow!("Consumer thread failed: {:?}", e))?;
    println!("Secret number: {}", secret);
    Ok(())
}
