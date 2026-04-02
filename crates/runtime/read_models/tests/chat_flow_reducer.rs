use deer_runtime_read_models::{
    reduce_chat_state, ChatAction, ChatDraftState, ClarificationState, OptimisticSendState,
};

#[test]
fn chat_reducer_tracks_prompt_optimistic_send_lifecycle() {
    let state = ChatDraftState::default();

    let state = reduce_chat_state(state, ChatAction::PromptSendStarted);

    assert_eq!(state.optimistic_send, OptimisticSendState::SendingPrompt);

    let state = reduce_chat_state(state, ChatAction::SendCompleted);

    assert_eq!(state.optimistic_send, OptimisticSendState::Idle);
}

#[test]
fn chat_reducer_requires_response_progress_before_clarification_can_resolve() {
    let state = ChatDraftState::default();

    let state = reduce_chat_state(state, ChatAction::ClarificationRequested);
    let state = reduce_chat_state(state, ChatAction::ClarificationResolved);

    assert_eq!(state.clarification_state, ClarificationState::Requested);
    assert_eq!(state.optimistic_send, OptimisticSendState::Idle);

    let state = reduce_chat_state(state, ChatAction::ClarificationResponseStarted);

    assert_eq!(state.clarification_state, ClarificationState::Responding);
    assert_eq!(
        state.optimistic_send,
        OptimisticSendState::SendingClarificationResponse
    );

    let state = reduce_chat_state(state, ChatAction::ClarificationResolved);

    assert_eq!(state.clarification_state, ClarificationState::None);
    assert_eq!(state.optimistic_send, OptimisticSendState::Idle);
}

#[test]
fn chat_reducer_tracks_uploads_and_stream_lifecycle() {
    let state = ChatDraftState::default();

    let state = reduce_chat_state(
        state,
        ChatAction::AddAttachment {
            attachment_id: "upload_1".into(),
        },
    );
    let state = reduce_chat_state(
        state,
        ChatAction::StreamStateChanged {
            state: "degraded".into(),
        },
    );

    assert_eq!(state.attachment_ids, vec!["upload_1".to_string()]);
    assert_eq!(state.stream_state.as_deref(), Some("degraded"));
}

#[test]
fn chat_reducer_clears_clarification_mode_when_prompt_send_starts() {
    let state = ChatDraftState {
        clarification_state: ClarificationState::Requested,
        ..Default::default()
    };

    let state = reduce_chat_state(state, ChatAction::PromptSendStarted);

    assert_eq!(state.clarification_state, ClarificationState::None);
    assert_eq!(state.optimistic_send, OptimisticSendState::SendingPrompt);
}

#[test]
fn chat_reducer_replaces_prompt_send_when_clarification_is_requested() {
    let state = ChatDraftState {
        optimistic_send: OptimisticSendState::SendingPrompt,
        ..Default::default()
    };

    let state = reduce_chat_state(state, ChatAction::ClarificationRequested);

    assert_eq!(state.clarification_state, ClarificationState::Requested);
    assert_eq!(state.optimistic_send, OptimisticSendState::Idle);
}
