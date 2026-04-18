use std::path::{Path, PathBuf};

/// A validated base directory for the entire system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BaseDir(PathBuf);

impl BaseDir {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn incoming(&self) -> IncomingDir {
        IncomingDir(self.0.join("incoming"))
    }

    pub fn vfs(&self) -> VfsDir {
        VfsDir(self.0.join("vfs"))
    }
}

/// The root directory for incoming raw data before Berg10 ingestion.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IncomingDir(PathBuf);

impl IncomingDir {
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn staging_db(&self) -> StagingDatabasePath {
        StagingDatabasePath(self.0.join("staging_area.redb"))
    }
}

/// The root directory for the Berg10 Virtual File System.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VfsDir(PathBuf);

impl VfsDir {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// The path to the redb staging database file.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StagingDatabasePath(PathBuf);

impl StagingDatabasePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}
