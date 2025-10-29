use std::time::{Duration, Instant};

use tokio::sync::{broadcast, mpsc};

use crate::{
    types::{AskCpu, SayCpu},
    CONFIG,
    errors::Error,
    notify
};

pub async fn ensure_cpu(req_tx: mpsc::Sender<AskCpu>, mut resp_rx: mpsc::Receiver<Vec<SayCpu>>, shutdown: broadcast::Sender<bool>) -> Result<(), Error>{

    let mut shutdown_rx = shutdown.subscribe();
    drop(shutdown);

    let config = CONFIG.get().unwrap();
    let (max_cpu_load, max_cpu_overload_duration, cpu_check_time_period) = 
    (config.max_cpu_load, config.max_cpu_overload_dur, config.cpu_check_time_period);

    let mut start_overload: Option<Instant> = None;
    let mut alerted = false;
    let mut overload_ongoing_for: Option<Duration> = None;

    loop{
        req_tx.send(AskCpu::Usage).await?;
        let ans =  resp_rx.recv().await;
        if ans.is_none(){
            eprint!("ERROR: Couldn't collect CPU data!");
            return Err(Error::CpuAns);
        }
        let ans = ans.unwrap();

        // Get average usage of all the cores
        let mut n = 0;
        let mut full_cpu = 0.0;
        for ans in ans{
            full_cpu += ans.usage;
            n += 1;
        }
        let usage = full_cpu / n as f32; // avg
        // println!("Cpu usage: {:?}%", usage);
        if usage > max_cpu_load{
            if start_overload.is_none(){
                // detected overload
                start_overload = Some(Instant::now());
                println!("CPU Overload Detected");
            }
            else if (start_overload.unwrap().elapsed() > max_cpu_overload_duration) && !alerted{
                // first alert
                notify::alert_cpu_overload(overload_ongoing_for);
                alerted = true;
                overload_ongoing_for = Some(start_overload.unwrap().elapsed());
            }
            else if start_overload.unwrap().elapsed() > max_cpu_overload_duration{
                // ongoing overload
                overload_ongoing_for = Some(start_overload.unwrap().elapsed());
                notify::alert_cpu_overload(overload_ongoing_for);
            }
        }
        else{
            start_overload = None;
            alerted = false;
            overload_ongoing_for = None
        }

        if let Ok(_) = shutdown_rx.try_recv(){
            break;
        }
        tokio::time::sleep(cpu_check_time_period).await;
    }
    drop(req_tx);
    Ok(())
}