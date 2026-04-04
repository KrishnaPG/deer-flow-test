use deer_storage_core::{ports::DurablePublisher, service::file_accepted_after_durable_publish};

struct TestPublisher;
struct FailingPublisher;

impl DurablePublisher for TestPublisher {
    fn publish_durable(&self, topic_name: &str, routing_key: &str) -> Result<(), &'static str> {
        if topic_name == "write-intent" && routing_key == "mission_7" {
            Ok(())
        } else {
            Err("unexpected durable publish route")
        }
    }
}

impl DurablePublisher for FailingPublisher {
    fn publish_durable(&self, _topic_name: &str, _routing_key: &str) -> Result<(), &'static str> {
        Err("publish failed")
    }
}

#[test]
fn file_accepted_means_durable_publish_succeeded() {
    let accepted =
        file_accepted_after_durable_publish(&TestPublisher, "write_7", "idem_7", "mission_7")
            .expect("durable publish should succeed");

    assert_eq!(accepted.topic_class, "write-intent");
    assert_eq!(accepted.routing_key, "mission_7");
}

#[test]
fn file_accepted_fails_when_durable_publish_fails() {
    let result =
        file_accepted_after_durable_publish(&FailingPublisher, "write_7", "idem_7", "mission_7");

    assert_eq!(result, Err("publish failed"));
}
