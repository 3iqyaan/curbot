use std::{time::{Duration}, sync::OnceLock};

use tokio::sync::{mpsc};

use crate::{
    errors::Error
};

mod resource;
mod termi;
mod save;
mod types;
mod errors;
mod notify;

use resource::{ask, check};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug)]
pub struct Config{
    max_cpu_load: f32,
    max_cpu_overload_dur: Duration,
    cpu_check_time_period: Duration,
    max_ram_load: f32,
    max_ram_overload_dur: Duration,
    ram_check_time_period: Duration
}

#[tokio::main]
async fn main() -> Result<(), Error>{
    CONFIG.set(Config{
        max_cpu_load: 30.0,
        max_cpu_overload_dur: Duration::from_secs(60),
        cpu_check_time_period: Duration::from_millis(500),
        max_ram_load: 50.0,
        max_ram_overload_dur: Duration::from_secs(60),
        ram_check_time_period: Duration::from_millis(750)
    }).unwrap();

    let (shutdown, _) = tokio::sync::broadcast::channel(64);

    let (req_tx, req_rx) = mpsc::channel(32);
    let (resp_tx, resp_rx) = mpsc::channel(8);

    let shutdown_ = shutdown.clone();
    let cpu_handle = tokio::spawn(async move{
        ask::cpu::main(req_rx, resp_tx, shutdown_).await
    });
    let shutdown_ = shutdown.clone();
    let ensure_cpu_handle = tokio::spawn(async move{
        check::cpu::ensure_cpu(req_tx, resp_rx, shutdown_).await
    });

    let (req_tx, req_rx) = mpsc::channel(32);
    let (resp_tx, resp_rx) = mpsc::channel(8);

    let shutdown_ = shutdown.clone();
    let mem_handle = tokio::spawn(async move{
        ask::memory::main(req_rx, resp_tx, shutdown_).await
    });
    let shutdown_ = shutdown.clone();
    let ensure_mem_handle = tokio::spawn(async move{
        check::memory::ensure_mem(req_tx, resp_rx, shutdown_).await
    });

    loop{
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                shutdown.send(true).unwrap();
                println!("Gracefully shutting down ...");
                ensure_cpu_handle.await??;
                cpu_handle.await??;
                ensure_mem_handle.await??;
                mem_handle.await??;
                break;
            }
        }
    }
    Ok(())
}



