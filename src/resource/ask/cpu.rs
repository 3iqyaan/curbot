use sysinfo::{CpuRefreshKind, Pid, RefreshKind, System};

use crate::{errors::Error, types::{AskCpu, SayCpu}};

pub fn main(ask: AskCpu) -> Result<Vec<SayCpu>, Error >{
    let mut sys = System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));

    sys.refresh_cpu_all();
    let mut ans = Vec::new();
    match ask{
        AskCpu::Usage => {
            for cpu in sys.cpus(){
                ans.push(SayCpu::Usage(cpu.cpu_usage()));
            }
            Ok(ans)
        },
        AskCpu::Proc(id) => {
            ans.push({
                let proc = sys.process(Pid::from_u32(id));
                SayCpu::Proc({
                    proc.map(|proc| (proc.accumulated_cpu_time(), proc.cpu_usage()))
                })
            });
            Ok(ans)
        },
        _ => todo!()
    } 
}