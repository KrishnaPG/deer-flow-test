use deer_pipeline_derivations::ArtifactShelfVm;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ArtifactShelfItemViewState {
    pub artifact_id: String,
    pub title: String,
    pub preview_action: Option<&'static str>,
    pub preview_available: bool,
    pub open_action: &'static str,
    pub download_action: &'static str,
    pub install_action: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ArtifactShelfViewState {
    pub item_count: usize,
    pub items: Vec<ArtifactShelfItemViewState>,
}

pub fn render_artifact_shelf_view(vm: &ArtifactShelfVm) -> ArtifactShelfViewState {
    let items: Vec<ArtifactShelfItemViewState> = vm
        .entries
        .iter()
        .filter(|entry| {
            entry.presentation_state == "presented" && entry.retrieval_mode == "mediated_pointer"
        })
        .map(|entry| ArtifactShelfItemViewState {
            artifact_id: entry.artifact_id.clone(),
            title: entry.title.clone(),
            preview_action: entry.preview_supported.then_some("preview_artifact"),
            preview_available: entry.preview_supported,
            open_action: "open_artifact",
            download_action: "download_artifact",
            install_action: "install_artifact",
        })
        .collect();

    ArtifactShelfViewState {
        item_count: items.len(),
        items,
    }
}
