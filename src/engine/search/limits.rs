#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchLimits
{
    pub depth: Option<u8>,
    pub movetime_ms: Option<u64>,
    pub infinite: bool,
}
