use deer_storage_core::boundary::livekit_bypass_note;
use deer_storage_core::diagnostics::InternalLifecycleEvent;

#[test]
fn livekit_media_bypass_stays_outside_normal_storage_ingress() {
    assert!(livekit_bypass_note().contains("landing zones"));
}

#[test]
fn internal_diagnostics_are_not_public_storage_facts() {
    let event = InternalLifecycleEvent::WriteStarted;
    assert_eq!(event.as_str(), "WriteStarted");
}
