use std::time::Duration;

pub fn alert_cpu_overload(dur: Option<Duration>){
    match dur{
        Some(dur) => {
            println!("CPU ON OVERLOAD for {:?}s", dur);
        }
        None => {
            println!("ALERT: CPU has been overloaded for longer than the threshold duration!");
        }
    }
}
pub fn alert_mem_overload(dur: Option<Duration>){
    match dur{
        Some(dur) => {
            println!("RAM ON OVERLOAD for {:?}s", dur);
        }
        None => {
            println!("ALERT: RAM has been overloaded for longer than the threshold duration!");
        }
    }
}