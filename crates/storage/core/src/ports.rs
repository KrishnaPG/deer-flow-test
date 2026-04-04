pub trait DurablePublisher {
    fn publish_durable(&self, topic_name: &str, routing_key: &str) -> Result<(), &'static str>;
}
