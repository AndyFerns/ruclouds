#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    IncreaseSpeed,
    DecreaseSpeed,
    DecreaseDensity,
    IncreaseDensity,
    CyclePalette,
    CycleWind,
    ToggleWarp,
    Reseed,
    ToggleStorm,
    Quit,
}
