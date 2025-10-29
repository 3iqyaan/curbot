use sysinfo::{CpuRefreshKind, MINIMUM_CPU_UPDATE_INTERVAL, Pid, RefreshKind, System};
use tokio::sync::{broadcast, mpsc};

use crate::{errors::Error, types::{AskCpu, SayCpu}};

pub async fn main(mut req_rx: mpsc::Receiver<AskCpu>, resp_tx: mpsc::Sender<Vec<SayCpu>>, shutdown: broadcast::Sender<bool>) -> Result<(), Error >{
    let mut sys = System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    let mut shutdown_rx = shutdown.subscribe();
    drop(shutdown);

    tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
    

    loop{
        sys.refresh_cpu_all();
        let mut ans = Vec::new();
        if let Ok(ask) = req_rx.try_recv(){
            match ask{
                AskCpu::All => {
                    for cpu in sys.cpus(){
                        ans.push(SayCpu{
                            name: cpu.name().to_string(),
                            brand: cpu.brand().to_string(),
                            freq: cpu.frequency(),
                            usage: cpu.cpu_usage(),
                            ..Default::default()
                        });
                    } 
                    resp_tx.send(ans).await?;
                },
                AskCpu::Usage => {
                    for cpu in sys.cpus(){
                        ans.push(SayCpu {
                            usage: cpu.cpu_usage(),
                            ..Default::default()
                        });
                    }
                    resp_tx.send(ans).await?;
                },
                AskCpu::Proc(id) => {
                    ans.push({
                        let proc = sys.process(Pid::from_u32(id));
                        SayCpu {
                            proc: match proc{
                                Some(proc) => Some((proc.accumulated_cpu_time(), proc.cpu_usage())),
                                None => None
                            },
                            ..Default::default()
                        }
                    });
                    resp_tx.send(ans).await?;
                },
                _ => todo!()
            }
            if let Ok(_) = shutdown_rx.try_recv(){
                break;
            }
            else{
                continue;
            }
        }
        else{
            tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL * 125 / 100).await;
        }
    }
    drop(resp_tx);
    Ok(())
}