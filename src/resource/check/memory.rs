use std::time::{Duration, Instant};

use tokio::sync::{broadcast, mpsc};

use crate::{
    types::{AskMem, SayMem},
    CONFIG,
    errors::Error,
    notify
};

pub async fn ensure_mem(req_tx: mpsc::Sender<AskMem>, mut resp_rx: mpsc::Receiver<SayMem>, shutdown: broadcast::Sender<bool>) -> Result<(), Error>{
    let mut shutdown_rx = shutdown.subscribe();
    drop(shutdown);
    
    // let ram_check_time_period = *RAM_check_TIME_PERIOD.lock().await;
    // let max_ram_load = *MAX_RAM_LOAD.lock().await;
    // let max_ram_overload_duration = *MAX_RAM_OVERLOAD_DURATION.lock().await;

    let config = CONFIG.get().unwrap();
    let (max_ram_load, max_ram_overload_duration, ram_check_time_period) = 
    (config.max_ram_load, config.max_ram_overload_dur, config.ram_check_time_period);


    let mut start_overload: Option<Instant> = None;
    let mut alerted = false;
    let mut overload_ongoing_for: Option<Duration> = None;

    loop {
        req_tx.send(AskMem::ARam).await?;
        let ans_aram =  resp_rx.recv().await;
        req_tx.send(AskMem::TRam).await?;
        let ans_tram = resp_rx.recv().await;

        if ans_aram.is_none(){
            eprint!("ERROR: Couldn't collect RAM data!");
            return Err(Error::RamAns);
        }
        if ans_tram.is_none(){
            eprint!("ERROR: Couldn't collect RAM data!");
            return Err(Error::RamAns);
        }
        // println!("Received ram data");

        let ans = ans_aram.unwrap();
        let ans_tram = ans_tram.unwrap();

        let usage = ((ans_tram.tram - ans.aram) as f64 / ans_tram.tram as f64 * 100.0) as f32;
        // println!("total ram: {:?}mb, free ram: {:?}mb, used ram: {:?}%", ans_tram.tram / 10_000, ans.aram / 10_000, usage);
        if usage > max_ram_load{
            if start_overload.is_none(){
                // detected overload
                start_overload = Some(Instant::now());
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
            start_overload = None;
            alerted = false;
            overload_ongoing_for = None
        }

        if let Ok(_) = shutdown_rx.try_recv(){
            break;
        }
        tokio::time::sleep(ram_check_time_period).await;
    }
    drop(req_tx);
    Ok(())
}