use deer_storage_core::admission::{AdmissionBudget, AdmissionController};

#[test]
fn rejects_when_safety_thresholds_are_exceeded() {
    let controller = AdmissionController::new(AdmissionBudget::new(1, 1024));
    assert!(controller.try_accept(2, 100).is_err());
}
