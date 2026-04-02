#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediatedPointer(String);

impl MediatedPointer {
    pub fn new(href: String) -> Result<Self, &'static str> {
        if href.starts_with("mediated://") {
            Ok(Self(href))
        } else {
            Err("artifact access must remain mediated")
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilePresenterInput {
    Preview {
        mime: String,
    },
    Pointer {
        mime: String,
        pointer: MediatedPointer,
    },
}

impl FilePresenterInput {
    pub fn mediated_pointer(mime: String, href: String) -> Result<Self, &'static str> {
        Ok(Self::Pointer {
            mime,
            pointer: MediatedPointer::new(href)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FilePresenterViewState {
    pub mode: String,
    pub mime: String,
    pub access_action: &'static str,
    pub href: Option<String>,
}

pub fn render_file_presenter_view(input: &FilePresenterInput) -> FilePresenterViewState {
    match input {
        FilePresenterInput::Preview { mime } => FilePresenterViewState {
            mode: "preview".into(),
            mime: mime.clone(),
            access_action: "inline_preview",
            href: None,
        },
        FilePresenterInput::Pointer { mime, pointer } => FilePresenterViewState {
            mode: "pointer".into(),
            mime: mime.clone(),
            access_action: "open_pointer",
            href: Some(pointer.as_str().to_string()),
        },
    }
}
