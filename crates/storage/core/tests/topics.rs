use deer_storage_core::topics::{route_topic, TopicClass};

#[test]
fn topic_routing_uses_stable_routing_keys() {
    let route = route_topic(TopicClass::WriteIntent, "mission_7");
    assert_eq!(route.topic_name, "write-intent");
    assert_eq!(route.routing_key, "mission_7");
}
