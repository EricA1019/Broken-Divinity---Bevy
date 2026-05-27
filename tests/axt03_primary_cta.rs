use broken_divinity::primary_cta::{
    AppSurface,
    CtaPolicy,
    PrimaryCta,
};

#[test]
fn menu_has_single_primary_cta() {
    let policy = CtaPolicy::default();
    let cta = policy.primary_for(AppSurface::Menu);

    assert_eq!(cta, PrimaryCta::StartRun);
}

#[test]
fn colony_has_single_primary_cta() {
    let policy = CtaPolicy::default();
    let cta = policy.primary_for(AppSurface::Colony);

    assert_eq!(cta, PrimaryCta::TravelToOverworld);
}

#[test]
fn overworld_has_single_primary_cta() {
    let policy = CtaPolicy::default();
    let cta = policy.primary_for(AppSurface::Overworld);

    assert_eq!(cta, PrimaryCta::EnterDungeon);
}
