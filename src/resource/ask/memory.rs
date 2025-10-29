use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, MemoryRefreshKind, Pid, RefreshKind, System};
use tokio::sync::{broadcast, mpsc};

use crate::{errors::Error, types::{AskMem, SayMem}};

pub async fn main(mut req_rx: mpsc::Receiver<AskMem>, resp_tx: mpsc::Sender<SayMem>, shutdown: broadcast::Sender<bool>) -> Result<(), Error>{
    let mut sys = System::new_with_specifics(RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()));
    let mut shutdown_rx = shutdown.subscribe();
    drop(shutdown);

    loop{
        if let Ok(ask) = req_rx.try_recv(){
            sys.refresh_memory();
            match ask{
                AskMem::ARam => {
                    resp_tx.send(SayMem{
                        aram: sys.available_memory(),
                        ..Default::default()
                    }).await?;
                }
                AskMem::FRam => {
                    resp_tx.send(SayMem{
                        fram: sys.free_memory(),
                        ..Default::default()
                    }).await?;
                }
                AskMem::TRam => {
                    resp_tx.send(SayMem{
                        tram: sys.total_memory(),
                        ..Default::default()
                    }).await?;
                }
                AskMem::TSwap => {
                    resp_tx.send(SayMem{
                        tswap: sys.total_swap(),
                        ..Default::default()
                    }).await?;
                }
                AskMem::FSwap => {
                    resp_tx.send(SayMem{
                        aram: sys.free_swap(),
                        ..Default::default()
                    }).await?;
                }
                AskMem::All => {
                    resp_tx.send(SayMem { 
                        tram: sys.total_memory(),
                        aram: sys.available_memory(),
                        fram: sys.free_memory(),
                        tswap: sys.total_swap(),
                        fswap: sys.free_swap(),
                        ..Default::default()
                    }).await?;
                }
                AskMem::Proc(id) => {
                    let proc = sys.process(Pid::from_u32(id));
                    resp_tx.send(SayMem {
                        proc: match proc{
                            Some(proc) => Some((proc.memory(), proc.virtual_memory())),
                            None => None
                        },
                        ..Default::default()
                    }).await?;
                }
            }
        }
        else{
            tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL * 2).await;
        }
        if let Ok(_) = shutdown_rx.try_recv(){
            break;
        }
    }
    drop(resp_tx);
    Ok(())
}