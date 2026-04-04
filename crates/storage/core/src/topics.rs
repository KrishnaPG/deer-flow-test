#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicClass {
    WriteIntent,
    ProgressLifecycle,
    DerivationTrigger,
    ControlIntent,
}

impl TopicClass {
    pub fn as_str(self) -> &'static str {
        match self {
            TopicClass::WriteIntent => "write-intent",
            TopicClass::ProgressLifecycle => "progress-lifecycle",
            TopicClass::DerivationTrigger => "derivation-trigger",
            TopicClass::ControlIntent => "control-intent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicRoute {
    pub topic_name: &'static str,
    pub routing_key: String,
}

pub fn route_topic(class: TopicClass, routing_key: &str) -> TopicRoute {
    TopicRoute {
        topic_name: class.as_str(),
        routing_key: routing_key.to_owned(),
    }
}
