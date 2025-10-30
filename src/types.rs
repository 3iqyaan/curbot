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

pub enum SayCpu{
    Name(String),
    Brand(String),
    Freq(u64),
    Usage(f32),
    Proc(Option<(u64, f32)>)
}

pub enum AskMem{
    TRam,
    ARam,
    FRam,
    TSwap,
    FSwap,
    Proc(u32),
    URam,
    All
}

pub enum SayMem{
    TRam(u64),
    ARam(u64),
    FRam(u64),
    TSwap(u64),
    FSwap(u64),
    Proc(Option<(u64, u64)>),
    URam(f32)
}