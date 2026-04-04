use deer_foundation_contracts::{FileAccepted, IdempotencyKey, LogicalWriteId};

use crate::{
    ports::DurablePublisher,
    topics::{route_topic, TopicClass},
};

pub fn file_accepted_after_durable_publish(
    publisher: &dyn DurablePublisher,
    logical_write_id: &str,
    idempotency_key: &str,
    routing_key: &str,
) -> Result<FileAccepted, &'static str> {
    let route = route_topic(TopicClass::WriteIntent, routing_key);
    publisher.publish_durable(&route.topic_name, &route.routing_key)?;

    Ok(FileAccepted {
        logical_write_id: LogicalWriteId::new(logical_write_id),
        idempotency_key: IdempotencyKey::new(idempotency_key),
        topic_class: route.topic_name.to_owned(),
        routing_key: route.routing_key,
    })
}
