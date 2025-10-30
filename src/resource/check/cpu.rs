use std::time::{Duration, Instant};

use tokio::sync::{broadcast};

use crate::{
    CONFIG, FAST_CPU_CHECK, SLOW_CPU_CHECK, errors::Error, notify, resource::ask, types::{AskCpu, SayCpu}
};

pub async fn check_cpu(shutdown: broadcast::Sender<bool>) -> Result<(), Error>{

    let mut shutdown_rx = shutdown.subscribe();
    drop(shutdown);

    let config = CONFIG.get().unwrap();
    let (max_cpu_load, max_cpu_overload_duration) = 
    (config.max_cpu_load, config.max_cpu_overload_dur);

    let mut cpu_check_time_period = SLOW_CPU_CHECK;
    let mut start_overload: Option<Instant> = None;
    let mut alerted = false;
    let mut overload_ongoing_for: Option<Duration> = None;

    loop{
        tokio::time::sleep(cpu_check_time_period).await;

        let ans = ask::cpu::main(AskCpu::Usage);
        if ans.is_err(){
            eprint!("ERROR: Couldn't collect CPU data!");
            return Err(Error::CpuAns);
        }
        let ans = ans.unwrap();

        // Get average usage of all the cores
        let mut n = 0;
        let mut full_cpu = 0.0;
        for ans in ans{
            if let SayCpu::Usage(usage) = ans{
                full_cpu += usage;
                n += 1;
            }
        }
        let usage = full_cpu / n as f32; // avg
        // println!("Cpu usage: {:?}%", usage);
        if usage > max_cpu_load{
            if start_overload.is_none(){
                // detected overload
                start_overload = Some(Instant::now());
                cpu_check_time_period = FAST_CPU_CHECK;
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
            // Usage is normal
            cpu_check_time_period = SLOW_CPU_CHECK;
            start_overload = None;
            alerted = false;
            overload_ongoing_for = None
        }

        if shutdown_rx.try_recv().is_ok(){
            break;
        }
    }
    drop(shutdown_rx);
    Ok(())
}