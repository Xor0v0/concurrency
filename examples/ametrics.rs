use std::{thread, time::Duration};

use anyhow::Result;
use concurrency::AmapMetrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    let metrics = AmapMetrics::new(&[
        "task.worker.0",
        "task.worker.1",
        "task.worker.2",
        "task.worker.3",
        "request.worker.0",
        "request.worker.1",
    ]);

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

fn task_worker(idx: usize, metrics: AmapMetrics) -> Result<()> {
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

fn request_worker(idx: usize, metrics: AmapMetrics) -> Result<()> {
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
