#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitManifest {
    id: String,
    members: Vec<String>,
    finalized: bool,
}

impl CommitManifest {
    pub fn new(id: &str, members: Vec<String>) -> Self {
        Self {
            id: id.to_owned(),
            members,
            finalized: false,
        }
    }

    pub fn mark_finalized(mut self) -> Self {
        self.finalized = true;
        self
    }

    pub fn is_visible(&self) -> bool {
        self.finalized
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
