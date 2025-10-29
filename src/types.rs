pub enum AskCpu{
    Init,
    Name,
    Brand,
    Freq,
    Usage,
    Proc(u32),
    All,
    Shutdown
}

#[derive(Default)]
pub struct SayCpu{
    pub name: String,
    pub brand: String,
    pub freq: u64,
    pub usage: f32,
    pub proc: Option<(u64, f32)>
}

pub enum AskMem{
    TRam,
    ARam,
    FRam,
    TSwap,
    FSwap,
    Proc(u32),
    All
}

#[derive(Default)]
pub struct SayMem{
    pub tram: u64,
    pub aram: u64,
    pub fram: u64,
    pub tswap: u64,
    pub fswap: u64,
    pub proc: Option<(u64, u64)>
}