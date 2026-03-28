from __future__ import annotations

import json
import logging
import mimetypes
import os
import sys
import traceback
import uuid
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


def now_iso() -> str:
    return datetime.now(UTC).isoformat()


REPO_ROOT = Path(os.environ.get("DEER_GUI_REPO_ROOT", Path(__file__).resolve().parents[3]))
HARNESS_ROOT = REPO_ROOT / "3rdParty" / "deer-flow" / "backend" / "packages" / "harness"
if str(HARNESS_ROOT) not in sys.path:
    sys.path.insert(0, str(HARNESS_ROOT))

from langchain_core.messages import AIMessage, HumanMessage, SystemMessage, ToolMessage  # noqa: E402

from deerflow.client import DeerFlowClient  # noqa: E402
from deerflow.config.paths import get_paths  # noqa: E402
from deerflow.models import create_chat_model  # noqa: E402


def _configure_logging() -> logging.Logger:
    """Configure logging from DEER_GUI_LOG env var.

    Levels: DEBUG, INFO (default), WARNING, ERROR, CRITICAL
    All output goes to stderr (stdout is reserved for JSON protocol).
    """
    level_name = os.environ.get("DEER_GUI_LOG", "INFO").upper()
    level = getattr(logging, level_name, logging.INFO)
    logging.basicConfig(
        level=level,
        stream=sys.stderr,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
        datefmt="%H:%M:%S%.3f",
    )
    log = logging.getLogger("deer_gui.bridge")
    log.setLevel(level)
    return log


logger = _configure_logging()


@dataclass
class Attachment:
    filename: str
    size: int | None = None
    path: str | None = None
    artifact_url: str | None = None


@dataclass
class ToolCall:
    name: str
    args: dict[str, Any]
    id: str | None = None


@dataclass
class Usage:
    input_tokens: int | None = None
    output_tokens: int | None = None
    total_tokens: int | None = None


@dataclass
class MessageRecord:
    id: str
    role: str
    content: str
    created_at: str
    name: str | None = None
    tool_calls: list[ToolCall] = field(default_factory=list)
    attachments: list[Attachment] = field(default_factory=list)
    usage: Usage | None = None


@dataclass
class TodoItem:
    content: str
    status: str


@dataclass
class ThreadRecord:
    thread_id: str
    title: str
    created_at: str
    updated_at: str
    messages: list[MessageRecord] = field(default_factory=list)
    artifacts: list[str] = field(default_factory=list)
    todos: list[TodoItem] = field(default_factory=list)
    suggestions: list[str] = field(default_factory=list)

    def summary(self) -> dict[str, Any]:
        return {
            "thread_id": self.thread_id,
            "title": self.title,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "message_count": len(self.messages),
            "artifacts": list(self.artifacts),
        }


class ThreadStore:
    def __init__(self, root: Path):
        self.root = root
        self.root.mkdir(parents=True, exist_ok=True)

    def _path(self, thread_id: str) -> Path:
        return self.root / f"{thread_id}.json"

    def create_thread(self) -> ThreadRecord:
        thread_id = str(uuid.uuid4())
        record = ThreadRecord(
            thread_id=thread_id,
            title="New thread",
            created_at=now_iso(),
            updated_at=now_iso(),
        )
        self.save(record)
        return record

    def get_thread(self, thread_id: str) -> ThreadRecord:
        path = self._path(thread_id)
        if not path.exists():
            record = ThreadRecord(
                thread_id=thread_id,
                title="New thread",
                created_at=now_iso(),
                updated_at=now_iso(),
            )
            self.save(record)
            return record
        data = json.loads(path.read_text(encoding="utf-8"))
        return ThreadRecord(
            thread_id=data["thread_id"],
            title=data.get("title", "New thread"),
            created_at=data.get("created_at", now_iso()),
            updated_at=data.get("updated_at", now_iso()),
            messages=[
                MessageRecord(
                    id=message["id"],
                    role=message["role"],
                    content=message.get("content", ""),
                    created_at=message.get("created_at", now_iso()),
                    name=message.get("name"),
                    tool_calls=[ToolCall(**tool_call) for tool_call in message.get("tool_calls", [])],
                    attachments=[Attachment(**attachment) for attachment in message.get("attachments", [])],
                    usage=Usage(**message["usage"]) if message.get("usage") else None,
                )
                for message in data.get("messages", [])
            ],
            artifacts=list(data.get("artifacts", [])),
            todos=[TodoItem(**todo) for todo in data.get("todos", [])],
            suggestions=list(data.get("suggestions", [])),
        )

    def save(self, record: ThreadRecord) -> None:
        record.updated_at = now_iso()
        self._path(record.thread_id).write_text(json.dumps(asdict(record), indent=2), encoding="utf-8")

    def list_threads(self) -> list[dict[str, Any]]:
        threads: list[dict[str, Any]] = []
        for path in self.root.glob("*.json"):
            try:
                record = self.get_thread(path.stem)
            except Exception:
                logger.warning("Failed to read thread store %s", path, exc_info=True)
                continue
            threads.append(record.summary())
        threads.sort(key=lambda item: item["updated_at"], reverse=True)
        return threads

    def delete_thread(self, thread_id: str) -> None:
        self._path(thread_id).unlink(missing_ok=True)


class Bridge:
    def __init__(self) -> None:
        gui_home = Path(os.environ.get("DEER_GUI_HOME", Path.home() / ".deer-gui"))
        logger.info("Initializing Bridge (gui_home=%s)", gui_home)
        self.store = ThreadStore(gui_home / "threads")

        # Ensure ~/.deer-flow exists for SQLite checkpointer and other data
        deer_flow_data_dir = Path.home() / ".deer-flow"
        deer_flow_data_dir.mkdir(parents=True, exist_ok=True)
        logger.info("Ensured deer-flow data dir exists: %s", deer_flow_data_dir)

        config_path = os.environ.get("DEER_FLOW_CONFIG_PATH")
        logger.info("Creating DeerFlowClient (config_path=%s)", config_path)
        self.client = DeerFlowClient(config_path=config_path)
        logger.info("DeerFlowClient initialized successfully")

    def emit(self, payload: dict[str, Any]) -> None:
        line = json.dumps(payload, ensure_ascii=True)
        logger.debug("emit -> %s", line[:500])
        sys.stdout.write(line + "\n")
        sys.stdout.flush()

    def emit_response(self, command_id: str, result: dict[str, Any]) -> None:
        self.emit({"kind": "response", "id": command_id, "ok": True, "result": result})

    def emit_error(self, command_id: str | None, message: str) -> None:
        logger.error("emit_error id=%s: %s", command_id, message)
        self.emit({"kind": "response", "id": command_id, "ok": False, "error": message})

    def emit_event(self, command_id: str, event: str, data: dict[str, Any]) -> None:
        logger.debug("emit_event: %s (id=%s)", event, command_id)
        self.emit({"kind": "event", "id": command_id, "event": event, "data": data})

    def handle(self, envelope: dict[str, Any]) -> None:
        command_id = envelope.get("id")
        command = envelope.get("command")
        payload = envelope.get("payload", {})
        logger.info("handle: command=%s id=%s", command, command_id)
        logger.debug("handle: payload=%s", json.dumps(payload, default=str)[:500])
        try:
            if command == "list_threads":
                threads = self.store.list_threads()
                logger.info("list_threads: returning %d threads", len(threads))
                self.emit_response(command_id, {"threads": threads})
            elif command == "get_thread":
                thread = self.store.get_thread(payload["thread_id"])
                logger.info("get_thread: %s (%d messages)", payload["thread_id"], len(thread.messages))
                self.emit_response(command_id, {"thread": asdict(thread)})
            elif command == "create_thread":
                thread = self.store.create_thread()
                logger.info("create_thread: %s", thread.thread_id)
                self.emit_response(command_id, {"created_thread": asdict(thread)})
            elif command == "rename_thread":
                thread = self.store.get_thread(payload["thread_id"])
                thread.title = payload["title"]
                self.store.save(thread)
                logger.info("rename_thread: %s -> %s", payload["thread_id"], payload["title"])
                self.emit_response(command_id, {"renamed_thread": thread.summary()})
            elif command == "delete_thread":
                thread_id = payload["thread_id"]
                self.store.delete_thread(thread_id)
                get_paths().delete_thread_dir(thread_id)
                logger.info("delete_thread: %s", thread_id)
                self.emit_response(command_id, {"deleted_thread_id": thread_id})
            elif command == "list_models":
                result = self.client.list_models()
                logger.info("list_models: returning %d models", len(result.get("models", [])))
                self.emit_response(command_id, result)
            elif command == "send_message":
                logger.info("send_message: thread=%s text_len=%d", payload.get("thread_id"), len(payload.get("text", "")))
                self.emit_response(command_id, {"accepted": True})
                self._send_message(command_id, payload)
            elif command == "resolve_artifact":
                artifact = self._resolve_artifact(payload["thread_id"], payload["virtual_path"])
                self.emit_response(command_id, {"artifact": artifact})
            else:
                logger.warning("Unknown command: %s", command)
                self.emit_error(command_id, f"Unknown command: {command}")
        except Exception as exc:
            logger.exception("Bridge command failed: command=%s id=%s", command, command_id)
            self.emit_error(command_id, str(exc))

    def _resolve_artifact(self, thread_id: str, virtual_path: str) -> dict[str, Any]:
        path = get_paths().resolve_virtual_path(thread_id, virtual_path)
        mime_type, _ = mimetypes.guess_type(path.name)
        return {
            "thread_id": thread_id,
            "virtual_path": virtual_path,
            "host_path": str(path),
            "mime_type": mime_type or "application/octet-stream",
        }

    def _send_message(self, command_id: str, payload: dict[str, Any]) -> None:
        thread_id = payload["thread_id"]
        text = payload.get("text", "")
        attachment_paths = [Path(path) for path in payload.get("attachments", [])]
        context = payload.get("context", {})
        logger.info("_send_message: thread=%s text_len=%d attachments=%d context=%s",
                     thread_id, len(text), len(attachment_paths), context)

        thread = self.store.get_thread(thread_id)
        logger.debug("Thread loaded: %d existing messages", len(thread.messages))

        uploaded_attachments: list[Attachment] = []
        if attachment_paths:
            logger.info("Uploading %d files", len(attachment_paths))
            upload_response = self.client.upload_files(thread_id, attachment_paths)
            for file_info in upload_response.get("files", []):
                uploaded_attachments.append(
                    Attachment(
                        filename=file_info.get("filename", "file"),
                        size=_safe_int(file_info.get("size")),
                        path=file_info.get("virtual_path"),
                        artifact_url=file_info.get("artifact_url"),
                    )
                )
            logger.info("Uploaded %d files", len(uploaded_attachments))

        user_message = MessageRecord(
            id=f"local-user-{uuid.uuid4()}",
            role="user",
            content=text,
            created_at=now_iso(),
            attachments=uploaded_attachments,
        )
        thread.messages.append(user_message)
        self.store.save(thread)
        self.emit_event(command_id, "message", {"thread_id": thread_id, "message": asdict(user_message)})

        model_name = context.get("model_name") or None
        mode = context.get("mode") or "thinking"
        reasoning_effort = context.get("reasoning_effort") or None
        thinking_enabled = mode != "flash"
        plan_mode = mode in {"pro", "ultra"}
        subagent_enabled = mode == "ultra"

        logger.info("Agent config: model=%s mode=%s thinking=%s plan=%s subagent=%s effort=%s",
                     model_name, mode, thinking_enabled, plan_mode, subagent_enabled, reasoning_effort)

        config = self.client._get_runnable_config(
            thread_id,
            model_name=model_name,
            thinking_enabled=thinking_enabled,
            plan_mode=plan_mode,
            subagent_enabled=subagent_enabled,
            recursion_limit=1000,
        )
        logger.debug("RunnableConfig: %s", config)

        logger.info("Ensuring agent is created...")
        self.client._ensure_agent(config)
        logger.info("Agent ready, starting stream")

        seen_message_ids = {message.id for message in thread.messages}
        usage_totals = Usage(input_tokens=0, output_tokens=0, total_tokens=0)
        state = {"messages": [HumanMessage(content=text)]}
        runtime_context = {
            "thread_id": thread_id,
            "thinking_enabled": thinking_enabled,
            "is_plan_mode": plan_mode,
            "subagent_enabled": subagent_enabled,
        }
        if reasoning_effort:
            runtime_context["reasoning_effort"] = reasoning_effort

        chunk_count = 0
        new_message_count = 0
        try:
            for chunk in self.client._agent.stream(state, config=config, context=runtime_context, stream_mode="values"):
                chunk_count += 1
                logger.debug("Stream chunk #%d: keys=%s", chunk_count, list(chunk.keys()))

                for message in chunk.get("messages", []):
                    if isinstance(message, HumanMessage):
                        continue
                    message_id = getattr(message, "id", None) or f"anon-{uuid.uuid4()}"
                    if message_id in seen_message_ids:
                        continue
                    seen_message_ids.add(message_id)
                    record = self._message_to_record(message_id, message)
                    thread.messages.append(record)
                    new_message_count += 1
                    logger.debug("New message: role=%s id=%s content_len=%d",
                                 record.role, record.id, len(record.content))
                    if record.usage:
                        usage_totals.input_tokens = (usage_totals.input_tokens or 0) + (record.usage.input_tokens or 0)
                        usage_totals.output_tokens = (usage_totals.output_tokens or 0) + (record.usage.output_tokens or 0)
                        usage_totals.total_tokens = (usage_totals.total_tokens or 0) + (record.usage.total_tokens or 0)
                    self.emit_event(command_id, "message", {"thread_id": thread_id, "message": asdict(record)})

                title = chunk.get("title") or thread.title
                thread.title = title
                thread.artifacts = list(chunk.get("artifacts", []))
                thread.todos = [
                    TodoItem(content=str(item.get("content", "")), status=str(item.get("status", "pending")))
                    for item in chunk.get("todos", []) or []
                    if isinstance(item, dict)
                ]
                self.store.save(thread)
                self.emit_event(
                    command_id,
                    "state",
                    {
                        "thread_id": thread_id,
                        "title": thread.title,
                        "artifacts": thread.artifacts,
                        "todos": [asdict(todo) for todo in thread.todos],
                    },
                )
        except Exception:
            logger.exception("Error during agent stream (thread=%s, chunks=%d)", thread_id, chunk_count)
            self.emit_event(command_id, "error", {"message": f"Stream error: {traceback.format_exc()}"})
            # Still emit done so the UI unlocks
            self.emit_event(command_id, "done", {"thread_id": thread_id, "usage": asdict(usage_totals)})
            return

        logger.info("Stream finished: %d chunks, %d new messages, usage=%s",
                     chunk_count, new_message_count, asdict(usage_totals))

        logger.info("Generating follow-up suggestions...")
        suggestions = self._generate_suggestions(thread, model_name)
        thread.suggestions = suggestions
        self.store.save(thread)
        logger.info("Generated %d suggestions", len(suggestions))
        self.emit_event(command_id, "suggestions", {"thread_id": thread_id, "suggestions": suggestions})
        self.emit_event(command_id, "done", {"thread_id": thread_id, "usage": asdict(usage_totals)})

    def _generate_suggestions(self, thread: ThreadRecord, model_name: str | None) -> list[str]:
        transcript_parts: list[str] = []
        for message in thread.messages[-8:]:
            if message.role not in {"user", "assistant"}:
                continue
            transcript_parts.append(f"{message.role.upper()}: {message.content}")
        if not transcript_parts:
            return []

        prompt = (
            "Generate exactly three short follow-up prompts for the user based on this transcript. "
            "Return each suggestion on its own line with no numbering.\n\n"
            + "\n".join(transcript_parts)
        )

        try:
            model = create_chat_model(name=model_name, thinking_enabled=False)
            response = model.invoke(
                [
                    SystemMessage(content="You write concise next-step prompts for a chat UI."),
                    HumanMessage(content=prompt),
                ]
            )
            text = self.client._extract_text(response.content)
        except Exception:
            logger.warning("Suggestion generation failed", exc_info=True)
            return []

        suggestions = [line.strip(" -\t") for line in text.splitlines() if line.strip()]
        deduped: list[str] = []
        for suggestion in suggestions:
            if suggestion not in deduped:
                deduped.append(suggestion)
            if len(deduped) == 3:
                break
        return deduped

    def _message_to_record(self, message_id: str, message: Any) -> MessageRecord:
        content = self.client._extract_text(getattr(message, "content", ""))
        if isinstance(message, AIMessage):
            usage = getattr(message, "usage_metadata", None) or {}
            return MessageRecord(
                id=message_id,
                role="assistant",
                content=content,
                created_at=now_iso(),
                tool_calls=[
                    ToolCall(name=tool_call["name"], args=tool_call.get("args", {}), id=tool_call.get("id"))
                    for tool_call in getattr(message, "tool_calls", [])
                ],
                usage=Usage(
                    input_tokens=_safe_int(usage.get("input_tokens")),
                    output_tokens=_safe_int(usage.get("output_tokens")),
                    total_tokens=_safe_int(usage.get("total_tokens")),
                )
                if usage
                else None,
            )
        if isinstance(message, ToolMessage):
            return MessageRecord(
                id=message_id,
                role="tool",
                content=content,
                created_at=now_iso(),
                name=getattr(message, "name", None),
            )
        return MessageRecord(
            id=message_id,
            role="event",
            content=content,
            created_at=now_iso(),
        )


def _safe_int(value: Any) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except (TypeError, ValueError):
        return None


def main() -> int:
    logger.info("Bridge starting (pid=%d, DEER_GUI_LOG=%s)",
                os.getpid(), os.environ.get("DEER_GUI_LOG", "INFO"))
    bridge = Bridge()
    logger.info("Bridge initialized, emitting ready signal")
    bridge.emit({"kind": "ready"})
    logger.info("Waiting for commands on stdin...")
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        logger.debug("stdin <- %s", line[:500])
        try:
            bridge.handle(json.loads(line))
        except Exception as exc:
            logger.exception("Fatal bridge error")
            bridge.emit_error(None, f"Fatal bridge error: {exc}")
    logger.info("stdin closed, bridge exiting")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
