#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSurface {
    Menu,
    Colony,
    Overworld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryCta {
    StartRun,
    TravelToOverworld,
    EnterDungeon,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CtaPolicy;

impl CtaPolicy {
    pub fn primary_for(&self, surface: AppSurface) -> PrimaryCta {
        match surface {
            AppSurface::Menu => PrimaryCta::StartRun,
            AppSurface::Colony => PrimaryCta::TravelToOverworld,
            AppSurface::Overworld => PrimaryCta::EnterDungeon,
        }
    }
}
