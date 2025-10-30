use sysinfo::{ MemoryRefreshKind, RefreshKind, System};

use crate::{errors::Error, types::{AskMem, SayMem}};

pub fn main(ask: AskMem) -> Result<SayMem, Error>{
    let mut sys = System::new_with_specifics(RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()));
    
    sys.refresh_memory();
    match ask{
        AskMem::URam => {
            let aram = sys.available_memory();
            let tram = sys.total_memory();
            Ok(SayMem::URam((tram - aram) as f32 / tram as f32))
        }
        _ => todo!()
    }

}