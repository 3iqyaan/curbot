use std::time::{Duration, Instant};

use tokio::sync::{broadcast};

use crate::{
    CONFIG, FAST_RAM_CHECK, SLOW_RAM_CHECK, errors::Error, notify, resource::ask, types::{AskMem, SayMem}
};

pub async fn ensure_mem(shutdown: broadcast::Sender<bool>) -> Result<(), Error>{
    let mut shutdown_rx = shutdown.subscribe();
    drop(shutdown);

    let config = CONFIG.get().unwrap();
    let (max_ram_load, max_ram_overload_duration) = 
    (config.max_ram_load, config.max_ram_overload_dur);

    let mut ram_check_time_period = SLOW_RAM_CHECK;
    let mut start_overload: Option<Instant> = None;
    let mut alerted = false;
    let mut overload_ongoing_for: Option<Duration> = None;

    loop {
        let res_usage = ask::memory::main(AskMem::URam);

        if res_usage.is_err(){
            eprint!("ERROR: Couldn't collect RAM data!");
            return Err(Error::RamAns);
        }
        let res_usage = res_usage.unwrap();
        // println!("Received ram data");

        if let SayMem::URam(usage) = res_usage{
            // println!("total ram: {:?}mb, free ram: {:?}mb, used ram: {:?}%", ans_tram.tram / 10_000, ans.aram / 10_000, usage);
            if usage > max_ram_load{
                if start_overload.is_none(){
                    // detected overload
                    start_overload = Some(Instant::now());
                    ram_check_time_period = FAST_RAM_CHECK;
                    println!("RAM Overload Detected");
                }
                else if (start_overload.unwrap().elapsed() > max_ram_overload_duration) && !alerted{
                    // first alert
                    notify::alert_mem_overload(overload_ongoing_for);
                    alerted = true;
                    overload_ongoing_for = Some(start_overload.unwrap().elapsed());
                }
                else if start_overload.unwrap().elapsed() > max_ram_overload_duration{
                    // ongoing overload
                    overload_ongoing_for = Some(start_overload.unwrap().elapsed());
                    notify::alert_mem_overload(overload_ongoing_for);
                }
            }
            else{
                // Usage is normal
                ram_check_time_period = SLOW_RAM_CHECK;
                start_overload = None;
                alerted = false;
                overload_ongoing_for = None
            }
        }
        

        if shutdown_rx.try_recv().is_ok(){
            break;
        }
        tokio::time::sleep(ram_check_time_period).await;
    }
    drop(shutdown_rx);
    Ok(())
}