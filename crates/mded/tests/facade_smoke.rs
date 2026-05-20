//! Phase 0.2 — alias-crate smoke test.
//!
//! Confirms `use mded::…` resolves to the same types as
//! `use mackesd_core::…`. Type identity is preserved because
//! mded re-exports `pub use mackesd_core::*;` rather than
//! wrapping types.

#[test]
fn mded_reexports_health_report_from_mackesd_core() {
    // HealthReport is a Phase 12.1.3 public type that's been
    // shipping in mackesd_core::health since Phase 12. If the
    // alias is set up correctly, mded::health::HealthReport is
    // the same type.
    fn type_id_of<T: 'static>(_: &T) -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }
    let mded_report = mded::health::HealthReport::empty();
    let mackesd_report = mackesd_core::health::HealthReport::empty();
    assert_eq!(type_id_of(&mded_report), type_id_of(&mackesd_report));
}

#[test]
fn mded_namespace_resolves_for_path_safety() {
    // Phase 2.5 — PathPolicy is one of the locked public types.
    let policy = mded::path_safety::PathPolicy::empty();
    assert_eq!(policy.roots().len(), 0);
}

#[test]
fn mded_namespace_resolves_for_orchestrator() {
    // Phase 2.6 — Orchestrator is one of the most recent
    // additions, exercises the re-export across newer code.
    let orch = mded::orchestrator::Orchestrator::new();
    assert!(orch.is_empty());
}
