use std::{time::{Duration}, sync::OnceLock, path::Path, fs};

use serde::{Deserialize, Serialize};

use crate::{
    errors::Error
};

mod resource;
mod termi;
mod save;
mod types;
mod errors;
mod notify;

use resource::{check};

pub const SLOW_CPU_CHECK: Duration = Duration::from_millis(1100);
pub const FAST_CPU_CHECK: Duration = Duration::from_millis(500);
pub const SLOW_RAM_CHECK: Duration = Duration::from_millis(1350);
pub const FAST_RAM_CHECK: Duration = Duration::from_millis(700);

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Config{
    enable_cpu_check: bool,
    enable_ram_check: bool,
    max_cpu_load: f32,
    #[serde(with = "humantime_serde")]
    max_cpu_overload_dur: Duration,
    max_ram_load: f32,
    #[serde(with = "humantime_serde")]
    max_ram_overload_dur: Duration,
}

#[tokio::main]
async fn main() -> Result<(), Error>{
    let cfg = parse_args(load_json());
    CONFIG.set(cfg.clone()).unwrap();

    let (shutdown, _) = tokio::sync::broadcast::channel(64);

    if cfg.enable_cpu_check{
        let shutdown_ = shutdown.clone();
        tokio::spawn(async move{
            check::cpu::check_cpu(shutdown_).await
        });
    }
    
    if cfg.enable_ram_check{
        let shutdown_ = shutdown.clone();
        tokio::spawn(async move{
            check::memory::ensure_mem(shutdown_).await
        });
    }
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown.send(true).unwrap();
        }
    }
    
    Ok(())
}

fn parse_args(mut cfg: Config) -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut override_default = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cpu-load" => {
                cfg.max_cpu_load = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--ram-load" => {
                cfg.max_ram_load = args[i + 1].parse().unwrap();
                i += 1;
            }
            
            "--no-cpu" => cfg.enable_cpu_check = false,
            "--no-ram" => cfg.enable_ram_check = false,
            "--override-default" => override_default = true,
            _ => {}
        }
        i += 1;
    }

    // rewrite JSON file
    if override_default {
        fs::write(
            config_path(),
            serde_json::to_string_pretty(&cfg).unwrap(),
        ).expect("Failed to write updated config");
        println!("Updated defaults written to {}", config_path());
    }
    cfg
}

fn load_json() -> Config {
    let path = config_path();
    if Path::new(path).exists() {
        let contents = fs::read_to_string(path).expect("Failed to read config file");
        serde_json::from_str(&contents).expect("Failed to parse config file")
    } else {
        let cfg = Config {
            enable_cpu_check: true,
            enable_ram_check: true,
            max_cpu_load: 75.0,
            max_cpu_overload_dur: Duration::from_secs(60),
            max_ram_load: 55.0,
            max_ram_overload_dur: Duration::from_secs(90),
        };
        fs::write(path, serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        cfg
    }
}

fn config_path() -> &'static str{
    "config.json"
}



