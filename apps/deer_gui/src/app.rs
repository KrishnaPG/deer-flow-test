use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Panel, RichText, ScrollArea, Stroke,
    TextEdit,
};
use log::{debug, error, info, warn};
use rfd::FileDialog;

use crate::bridge::{BridgeClient, BridgeEvent, BridgeResponse};
use crate::models::{ArtifactInfo, ChatMessage, ModelInfo, ThreadRecord, ThreadSummary, Usage};

pub struct DeerGuiApp {
    bridge: Option<BridgeClient>,
    pending_requests: HashMap<String, PendingRequest>,
    threads: Vec<ThreadSummary>,
    thread_cache: HashMap<String, ThreadRecord>,
    selected_thread_id: Option<String>,
    composer: String,
    attachments: Vec<PathBuf>,
    models: Vec<ModelInfo>,
    selected_model: Option<String>,
    selected_mode: Mode,
    status: String,
    rename_buffer: String,
    artifact_preview: Option<ArtifactPreview>,
    usage_by_thread: HashMap<String, Usage>,
    streaming_thread_id: Option<String>,
    /// When true, auto-send the composer content after a thread is created.
    queued_send: bool,
}

#[derive(Debug, Clone)]
enum PendingRequest {
    Threads,
    Thread,
    CreatedThread,
    Models,
    Rename,
    Delete,
    SendMessage,
    ResolveArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flash,
    Thinking,
    Pro,
    Ultra,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Flash => "flash",
            Self::Thinking => "thinking",
            Self::Pro => "pro",
            Self::Ultra => "ultra",
        }
    }

    fn reasoning_effort(self) -> Option<&'static str> {
        match self {
            Self::Flash => None,
            Self::Thinking => Some("low"),
            Self::Pro => Some("medium"),
            Self::Ultra => Some("high"),
        }
    }
}

struct ArtifactPreview {
    info: ArtifactInfo,
    text_preview: Option<String>,
}

impl DeerGuiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let bridge = BridgeClient::spawn();
        let mut app = Self {
            bridge: None,
            pending_requests: HashMap::new(),
            threads: Vec::new(),
            thread_cache: HashMap::new(),
            selected_thread_id: None,
            composer: String::new(),
            attachments: Vec::new(),
            models: Vec::new(),
            selected_model: None,
            selected_mode: Mode::Thinking,
            status: String::new(),
            rename_buffer: String::new(),
            artifact_preview: None,
            usage_by_thread: HashMap::new(),
            streaming_thread_id: None,
            queued_send: false,
        };

        match bridge {
            Ok(bridge) => {
                info!("Bridge spawned successfully, requesting threads and models");
                app.status = "Bridge ready. Loading threads and models...".to_string();
                if let Ok(id) = bridge.list_threads() {
                    app.pending_requests.insert(id, PendingRequest::Threads);
                }
                if let Ok(id) = bridge.list_models() {
                    app.pending_requests.insert(id, PendingRequest::Models);
                }
                app.bridge = Some(bridge);
            }
            Err(err) => {
                error!("Failed to spawn bridge: {err}");
                app.status = err;
            }
        }

        app
    }

    fn process_bridge_events(&mut self, ctx: &egui::Context) {
        let Some(bridge) = &self.bridge else {
            return;
        };

        let events: Vec<_> = bridge.events().try_iter().collect();

        for event in events {
            match event {
                BridgeEvent::Ready => {
                    info!("Received BridgeEvent::Ready");
                    self.status = "Python bridge started".to_string();
                }
                BridgeEvent::Response {
                    request_id,
                    response,
                } => {
                    debug!(
                        "Received BridgeEvent::Response id={request_id:?} type={:?}",
                        std::mem::discriminant(&response)
                    );
                    self.handle_response(request_id, response);
                }
                BridgeEvent::StreamMessage { thread_id, message } => {
                    debug!(
                        "Received BridgeEvent::StreamMessage thread={thread_id} role={} len={}",
                        message.role,
                        message.content.len()
                    );
                    if let Some(thread) = self.thread_cache.get_mut(&thread_id) {
                        thread.messages.push(message);
                        thread.updated_at = now_string();
                    } else {
                        warn!("StreamMessage for unknown thread {thread_id}");
                    }
                    self.status = format!("Streaming reply in {thread_id}");
                    ctx.request_repaint();
                }
                BridgeEvent::State { state } => {
                    debug!(
                        "Received BridgeEvent::State thread={} title={:?}",
                        state.thread_id, state.title
                    );
                    if let Some(thread) = self.thread_cache.get_mut(&state.thread_id) {
                        if let Some(title) = state.title {
                            thread.title = title.clone();
                            if let Some(summary) = self
                                .threads
                                .iter_mut()
                                .find(|t| t.thread_id == state.thread_id)
                            {
                                summary.title = title;
                            }
                        }
                        thread.artifacts = state.artifacts.clone();
                        thread.todos = state.todos;
                        thread.updated_at = now_string();
                    }
                }
                BridgeEvent::Suggestions {
                    thread_id,
                    suggestions,
                } => {
                    debug!(
                        "Received BridgeEvent::Suggestions thread={thread_id} count={}",
                        suggestions.len()
                    );
                    if let Some(thread) = self.thread_cache.get_mut(&thread_id) {
                        thread.suggestions = suggestions;
                    }
                }
                BridgeEvent::Done { thread_id, usage } => {
                    info!("Received BridgeEvent::Done thread={thread_id} usage={usage:?}");
                    self.streaming_thread_id = None;
                    if let Some(usage) = usage {
                        self.usage_by_thread.insert(thread_id.clone(), usage);
                    }
                    self.status = format!("Completed response for {thread_id}");
                    if let Some(summary) =
                        self.threads.iter_mut().find(|t| t.thread_id == thread_id)
                    {
                        summary.updated_at = now_string();
                        if let Some(thread) = self.thread_cache.get(&thread_id) {
                            summary.message_count = thread.messages.len();
                            summary.artifacts = thread.artifacts.clone();
                            if !thread.title.is_empty() {
                                summary.title = thread.title.clone();
                            }
                        }
                    }
                }
                BridgeEvent::Error { message } => {
                    error!("Received BridgeEvent::Error: {message}");
                    self.streaming_thread_id = None;
                    self.status = message;
                }
                BridgeEvent::BridgeExited => {
                    warn!("Received BridgeEvent::BridgeExited");
                    self.status = "Python bridge exited".to_string();
                }
            }
        }
    }

    fn handle_response(&mut self, request_id: Option<String>, response: BridgeResponse) {
        let pending = request_id
            .as_ref()
            .and_then(|request_id| self.pending_requests.remove(request_id));

        debug!(
            "handle_response: pending={:?} response_type={:?}",
            pending,
            std::mem::discriminant(&response)
        );

        match response {
            BridgeResponse::Threads(threads) => {
                info!("Loaded {} threads", threads.len());
                self.threads = threads;
                if self.selected_thread_id.is_none() {
                    if let Some(thread) = self.threads.first() {
                        self.selected_thread_id = Some(thread.thread_id.clone());
                        self.rename_buffer = thread.title.clone();
                        self.fetch_thread(thread.thread_id.clone());
                    }
                }
            }
            BridgeResponse::Thread(thread) => {
                info!(
                    "Loaded thread {} ({} messages)",
                    thread.thread_id,
                    thread.messages.len()
                );
                self.rename_buffer = thread.title.clone();
                if self.selected_thread_id.is_none() {
                    self.selected_thread_id = Some(thread.thread_id.clone());
                }
                self.thread_cache.insert(thread.thread_id.clone(), thread);
            }
            BridgeResponse::CreatedThread(thread) => {
                info!("Created thread {}", thread.thread_id);
                let summary = ThreadSummary {
                    thread_id: thread.thread_id.clone(),
                    title: thread.title.clone(),
                    created_at: thread.created_at.clone(),
                    updated_at: thread.updated_at.clone(),
                    message_count: thread.messages.len(),
                    artifacts: thread.artifacts.clone(),
                };
                self.threads.insert(0, summary);
                self.rename_buffer = thread.title.clone();
                self.selected_thread_id = Some(thread.thread_id.clone());
                self.thread_cache.insert(thread.thread_id.clone(), thread);

                // If a send was queued (user hit Send with no thread), fire it now.
                if self.queued_send {
                    self.queued_send = false;
                    info!("Auto-sending queued message after thread creation");
                    self.send_message();
                }
            }
            BridgeResponse::Models(models) => {
                info!("Loaded {} models", models.len());
                self.models = models;
                if self.selected_model.is_none() {
                    self.selected_model = self.models.first().map(|model| model.name.clone());
                }
            }
            BridgeResponse::Renamed(summary) => {
                if let Some(thread) = self.thread_cache.get_mut(&summary.thread_id) {
                    thread.title = summary.title.clone();
                    thread.updated_at = summary.updated_at.clone();
                }
                if let Some(existing) = self
                    .threads
                    .iter_mut()
                    .find(|item| item.thread_id == summary.thread_id)
                {
                    *existing = summary;
                }
            }
            BridgeResponse::Deleted(thread_id) => {
                self.threads.retain(|thread| thread.thread_id != thread_id);
                self.thread_cache.remove(&thread_id);
                if self.selected_thread_id.as_deref() == Some(thread_id.as_str()) {
                    self.selected_thread_id =
                        self.threads.first().map(|thread| thread.thread_id.clone());
                    if let Some(thread_id) = self.selected_thread_id.clone() {
                        self.fetch_thread(thread_id);
                    }
                }
            }
            BridgeResponse::Artifact(info) => {
                let text_preview = std::fs::read_to_string(&info.host_path).ok();
                self.artifact_preview = Some(ArtifactPreview { info, text_preview });
            }
            BridgeResponse::Ack | BridgeResponse::Raw(_) => {}
        }

        if let Some(pending) = pending {
            match pending {
                PendingRequest::Threads
                | PendingRequest::Thread
                | PendingRequest::CreatedThread
                | PendingRequest::Models
                | PendingRequest::Rename
                | PendingRequest::Delete
                | PendingRequest::SendMessage
                | PendingRequest::ResolveArtifact => {}
            }
        }
    }

    fn fetch_thread(&mut self, thread_id: String) {
        if let Some(bridge) = &self.bridge {
            if let Ok(id) = bridge.get_thread(&thread_id) {
                self.pending_requests.insert(id, PendingRequest::Thread);
            }
        }
    }

    fn create_thread(&mut self) {
        if let Some(bridge) = &self.bridge {
            if let Ok(id) = bridge.create_thread() {
                self.pending_requests
                    .insert(id, PendingRequest::CreatedThread);
            }
        }
    }

    fn send_message(&mut self) {
        let Some(thread_id) = self.selected_thread_id.clone() else {
            info!("send_message: no thread selected, auto-creating one");
            self.queued_send = true;
            self.create_thread();
            return;
        };
        if self.composer.trim().is_empty() && self.attachments.is_empty() {
            debug!("send_message: empty composer and no attachments, ignoring");
            return;
        }
        if self.streaming_thread_id.is_some() {
            warn!("send_message: already streaming, ignoring");
            self.status = "Wait for the current response to finish".to_string();
            return;
        }

        let Some(bridge) = &self.bridge else {
            error!("send_message: no bridge available");
            return;
        };
        let text = self.composer.trim().to_string();
        info!(
            "Sending message: thread={thread_id} mode={} model={:?} text_len={}",
            self.selected_mode.as_str(),
            self.selected_model,
            text.len()
        );
        let attachments = self.attachments.clone();
        let model_name = self.selected_model.as_deref();
        let mode = self.selected_mode.as_str();
        let reasoning_effort = self.selected_mode.reasoning_effort();
        match bridge.send_message(
            &thread_id,
            &text,
            &attachments,
            model_name,
            mode,
            reasoning_effort,
        ) {
            Ok(id) => {
                debug!("send_message dispatched: request_id={id}");
                self.pending_requests
                    .insert(id, PendingRequest::SendMessage);
                self.streaming_thread_id = Some(thread_id.clone());
                self.status = format!("Sending message to {thread_id}");
                self.attachments.clear();
                self.composer.clear();
            }
            Err(err) => {
                error!("send_message failed: {err}");
                self.status = err;
            }
        }
    }

    fn rename_selected_thread(&mut self) {
        let Some(thread_id) = self.selected_thread_id.clone() else {
            return;
        };
        let title = self.rename_buffer.trim().to_string();
        if title.is_empty() {
            return;
        }
        if let Some(bridge) = &self.bridge {
            if let Ok(id) = bridge.rename_thread(&thread_id, &title) {
                self.pending_requests.insert(id, PendingRequest::Rename);
            }
        }
    }

    fn delete_selected_thread(&mut self) {
        let Some(thread_id) = self.selected_thread_id.clone() else {
            return;
        };
        if let Some(bridge) = &self.bridge {
            if let Ok(id) = bridge.delete_thread(&thread_id) {
                self.pending_requests.insert(id, PendingRequest::Delete);
            }
        }
    }

    fn resolve_artifact(&mut self, virtual_path: String) {
        let Some(thread_id) = self.selected_thread_id.clone() else {
            return;
        };
        if let Some(bridge) = &self.bridge {
            if let Ok(id) = bridge.resolve_artifact(&thread_id, &virtual_path) {
                self.pending_requests
                    .insert(id, PendingRequest::ResolveArtifact);
            }
        }
    }

    fn selected_thread(&self) -> Option<&ThreadRecord> {
        self.selected_thread_id
            .as_ref()
            .and_then(|thread_id| self.thread_cache.get(thread_id))
    }

    fn apply_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = Color32::from_rgb(246, 240, 231);
        visuals.extreme_bg_color = Color32::from_rgb(255, 252, 247);
        visuals.window_fill = Color32::from_rgb(255, 251, 246);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(249, 244, 236);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(242, 235, 224);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(232, 224, 210);
        visuals.widgets.active.bg_fill = Color32::from_rgb(220, 209, 193);
        visuals.selection.bg_fill = Color32::from_rgb(182, 98, 58);
        visuals.hyperlink_color = Color32::from_rgb(156, 83, 47);
        visuals.override_text_color = Some(Color32::from_rgb(47, 38, 31));
        ctx.set_visuals(visuals);

        let mut style = (*ctx.global_style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.visuals.window_corner_radius = CornerRadius::same(16);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(14);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(14);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(14);
        ctx.set_global_style(style);
    }
}

impl eframe::App for DeerGuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Self::apply_theme(ui.ctx());
        self.process_bridge_events(ui.ctx());

        Panel::top("top_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Deer GUI").size(26.0).strong());
                ui.label("Embedded Deer Flow desktop chat");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(&self.status).italics());
                });
            });
        });

        Panel::left("threads")
            .resizable(true)
            .default_size(250.0)
            .show_inside(ui, |ui| {
                Frame::default()
                    .fill(Color32::from_rgb(254, 249, 243))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Threads");
                            if ui.button("New").clicked() {
                                self.create_thread();
                            }
                        });
                        ui.separator();
                        ScrollArea::vertical().show(ui, |ui| {
                            for thread in self.threads.clone() {
                                let selected = self.selected_thread_id.as_deref()
                                    == Some(thread.thread_id.as_str());
                                let response = ui.add_sized(
                                    [ui.available_width(), 60.0],
                                    egui::Button::new(
                                        RichText::new(format!(
                                            "{}\n{} msgs",
                                            thread.title, thread.message_count
                                        ))
                                        .size(14.0),
                                    )
                                    .selected(selected),
                                );
                                if response.clicked() {
                                    self.selected_thread_id = Some(thread.thread_id.clone());
                                    self.rename_buffer = thread.title.clone();
                                    self.fetch_thread(thread.thread_id.clone());
                                }
                            }
                        });
                    });
            });

        Panel::right("artifacts")
            .resizable(true)
            .default_size(260.0)
            .show_inside(ui, |ui| {
                Frame::default()
                    .fill(Color32::from_rgb(255, 249, 244))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.heading("Artifacts");
                        ui.separator();
                        let artifact_list = self
                            .selected_thread()
                            .map(|thread| thread.artifacts.clone())
                            .unwrap_or_default();
                        ScrollArea::vertical().show(ui, |ui| {
                            for artifact in artifact_list {
                                if ui.button(artifact.clone()).clicked() {
                                    self.resolve_artifact(artifact);
                                }
                            }
                        });
                        ui.separator();
                        if let Some(preview) = &self.artifact_preview {
                            ui.label(RichText::new(preview.info.virtual_path.clone()).strong());
                            ui.label(format!("Mime: {}", preview.info.mime_type));
                            if ui.button("Open externally").clicked() {
                                let _ = opener::open(&preview.info.host_path);
                            }
                            if let Some(text) = &preview.text_preview {
                                ui.separator();
                                ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                                    ui.monospace(text);
                                });
                            }
                        }
                    });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let thread_title = self
                .selected_thread()
                .map(|thread| thread.title.clone())
                .unwrap_or_else(|| "New thread".to_string());

            Frame::default()
                .fill(Color32::from_rgb(255, 252, 248))
                .corner_radius(CornerRadius::same(20))
                .stroke(Stroke::new(1.0, Color32::from_rgb(222, 210, 196)))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(thread_title);
                        if let Some(thread_id) = &self.selected_thread_id {
                            ui.label(RichText::new(thread_id).small().weak());
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Delete").clicked() {
                                self.delete_selected_thread();
                            }
                            if ui.button("Rename").clicked() {
                                self.rename_selected_thread();
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Title");
                        ui.add(TextEdit::singleline(&mut self.rename_buffer).desired_width(280.0));
                    });

                    ui.separator();

                    ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        if let Some(thread) = self.selected_thread() {
                            for message in &thread.messages {
                                render_message(ui, message);
                            }

                            if !thread.todos.is_empty() {
                                ui.separator();
                                ui.label(RichText::new("Task Board").strong());
                                for todo in &thread.todos {
                                    ui.label(format!("{} - {}", todo.status, todo.content));
                                }
                            }

                            if !thread.suggestions.is_empty() {
                                ui.separator();
                                ui.label(RichText::new("Follow-ups").strong());
                                let suggestions = thread.suggestions.clone();
                                for suggestion in &suggestions {
                                    if ui.button(suggestion).clicked() {
                                        self.composer = suggestion.clone();
                                    }
                                }
                            }
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(120.0);
                                ui.heading("Start a Deer Flow conversation");
                                ui.label("Create a thread, pick a mode, and send a prompt.");
                            });
                        }
                    });

                    ui.separator();

                    ui.horizontal_wrapped(|ui| {
                        ui.label("Model");
                        egui::ComboBox::from_id_salt("model_picker")
                            .selected_text(
                                self.selected_model
                                    .clone()
                                    .unwrap_or_else(|| "auto".to_string()),
                            )
                            .show_ui(ui, |ui| {
                                for model in &self.models {
                                    ui.selectable_value(
                                        &mut self.selected_model,
                                        Some(model.name.clone()),
                                        model
                                            .display_name
                                            .clone()
                                            .unwrap_or_else(|| model.name.clone()),
                                    );
                                }
                            });

                        ui.label("Mode");
                        for mode in [Mode::Flash, Mode::Thinking, Mode::Pro, Mode::Ultra] {
                            ui.selectable_value(&mut self.selected_mode, mode, mode.as_str());
                        }
                    });

                    ui.add_sized(
                        [ui.available_width(), 120.0],
                        TextEdit::multiline(&mut self.composer)
                            .hint_text(
                                "Ask Deer Flow to research, reason, code, or inspect files...",
                            )
                            .desired_rows(5),
                    );

                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Attach files").clicked() {
                            if let Some(files) = FileDialog::new().pick_files() {
                                self.attachments.extend(files);
                            }
                        }
                        for attachment in &self.attachments {
                            let name = attachment
                                .file_name()
                                .map(|name| name.to_string_lossy().to_string())
                                .unwrap_or_else(|| attachment.display().to_string());
                            ui.label(RichText::new(name).small());
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Send").clicked() {
                            self.send_message();
                        }
                        if let Some(thread_id) = &self.selected_thread_id {
                            if let Some(usage) = self.usage_by_thread.get(thread_id) {
                                let total = usage.total_tokens.unwrap_or_default();
                                ui.label(format!("Total tokens: {total}"));
                            }
                        }
                    });
                });
        });
    }
}

fn render_message(ui: &mut egui::Ui, message: &ChatMessage) {
    let (fill, label) = match message.role.as_str() {
        "user" => (Color32::from_rgb(240, 221, 201), "You"),
        "assistant" => (Color32::from_rgb(228, 235, 217), "Deer Flow"),
        "tool" => (
            Color32::from_rgb(229, 231, 239),
            message.name.as_deref().unwrap_or("Tool"),
        ),
        _ => (Color32::from_rgb(239, 236, 232), "Event"),
    };

    Frame::default()
        .fill(fill)
        .corner_radius(CornerRadius::same(16))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).strong());
                ui.label(RichText::new(format_time(&message.created_at)).small());
            });

            if !message.content.is_empty() {
                ui.label(&message.content);
            }

            if !message.attachments.is_empty() {
                ui.separator();
                for attachment in &message.attachments {
                    ui.label(format!("Attachment: {}", attachment.filename));
                }
            }

            if !message.tool_calls.is_empty() {
                ui.separator();
                for tool_call in &message.tool_calls {
                    ui.monospace(format!("{} {}", tool_call.name, tool_call.args));
                }
            }
        });
    ui.add_space(8.0);
}

fn format_time(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&Local).format("%b %d %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn now_string() -> String {
    chrono::Utc::now().to_rfc3339()
}
