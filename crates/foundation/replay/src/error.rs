use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("event sequence must be strictly increasing: last={last}, next={next}")]
    NonMonotonicSequence { last: u64, next: u64 },

    #[error("failed to read replay fixture {path}: {source}")]
    FixtureRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse replay fixture {path}: {source}")]
    FixtureParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}
