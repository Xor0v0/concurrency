use std::{thread, time::Duration};

use anyhow::Result;
use concurrency::Metrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    // mutltithreading access Metrics instance
    let metrics = Metrics::new();

    println!("metrics: {}", metrics);

    for idx in 0..M {
        task_worker(idx, metrics.clone())?;
    }

    for idx in 0..N {
        request_worker(idx, metrics.clone())?;
    }

    loop {
        let mut rng = rand::rng();
        thread::sleep(Duration::from_millis(rng.random_range(1000..2000)));
        println!("metrics: {}", metrics);
    }
}

fn task_worker(idx: usize, metrics: Metrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::rng();
            thread::sleep(Duration::from_millis(rng.random_range(50..1000)));
            metrics.inc(format!("task.worker.{}", idx))?;
        }

        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    Ok(())
}

fn request_worker(idx: usize, metrics: Metrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::rng();
            thread::sleep(Duration::from_millis(rng.random_range(50..1000)));
            metrics.inc(format!("request.worker.{}", idx))?;
        }

        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    Ok(())
}
