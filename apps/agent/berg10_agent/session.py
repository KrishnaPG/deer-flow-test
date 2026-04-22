"""In-memory session management for WebSocket connections."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any
from uuid import uuid4


@dataclass
class Session:
    """A single WebSocket session with conversation state."""

    session_id: str = field(default_factory=lambda: str(uuid4()))
    history: list[dict[str, Any]] = field(default_factory=list)
    cancelled: bool = False
    metadata: dict[str, Any] = field(default_factory=dict)
    current_models: dict[str, str] = field(default_factory=dict)  # mode -> model_id


class SessionManager:
    """Manages active sessions keyed by session ID."""

    def __init__(self) -> None:
        self._sessions: dict[str, Session] = {}

    def create(self) -> Session:
        session = Session()
        self._sessions[session.session_id] = session
        return session

    def get(self, session_id: str) -> Session | None:
        return self._sessions.get(session_id)

    def remove(self, session_id: str) -> None:
        self._sessions.pop(session_id, None)

    @property
    def count(self) -> int:
        return len(self._sessions)

    def list_ids(self) -> list[str]:
        return list(self._sessions.keys())
