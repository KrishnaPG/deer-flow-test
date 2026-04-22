"""
Berg10 Agent - Centralized constants and enums.

All message types, tool names, configuration keys, and protocol constants
defined here for easy modification and extension.
"""

from enum import Enum, auto
from typing import Final


# =============================================================================
# Message Types (WebSocket/JSON-RPC)
# =============================================================================


class MessageType(str, Enum):
    """WebSocket message types for client-server communication."""

    # Client → Server
    MESSAGE = "message"  # User input message
    INTERRUPT = "interrupt"  # Cancel current operation
    ACK = "ack"  # Acknowledge receipt
    CANCEL = "cancel"  # Cancel specific request

    # Server → Client
    CHUNK = "chunk"  # Streaming LLM token
    TOOL = "tool"  # Tool execution status
    ERROR = "error"  # Error response
    DONE = "done"  # Operation complete
    STREAM_START = "stream_start"  # Streaming begins
    STREAM_END = "stream_end"  # Streaming ends


# =============================================================================
# Tool Names
# =============================================================================


class ToolName(str, Enum):
    """Available tool names for agent execution."""

    LIST_FILES = "list_files"
    GREP_SEARCH = "grep_search"
    READ_FILE = "read_file"
    PATCH_FILE = "patch_file"
    RUN_COMMAND = "run_command"


# =============================================================================
# JSON-RPC Protocol
# =============================================================================


class JsonRpcVersion(str, Enum):
    """JSON-RPC protocol versions."""

    V2_0 = "2.0"
    V3_0 = "3.0"


DEFAULT_JSON_RPC_VERSION: Final[str] = JsonRpcVersion.V3_0.value


class JsonRpcMethod(str, Enum):
    """JSON-RPC method names."""

    # Lifecycle
    INITIALIZE = "initialize"
    SHUTDOWN = "shutdown"

    # Messaging
    SEND_MESSAGE = "send_message"
    STREAM_MESSAGE = "stream_message"

    # Tools
    TOOL_CALL = "tool/call"
    TOOL_CANCEL = "tool/cancel"


# =============================================================================
# Error Codes (JSON-RPC 3.0 + Custom)
# =============================================================================


class ErrorCode(int, Enum):
    """JSON-RPC and application error codes."""

    # JSON-RPC Standard
    PARSE_ERROR = -32700
    INVALID_REQUEST = -32600
    METHOD_NOT_FOUND = -32601
    INVALID_PARAMS = -32602
    INTERNAL_ERROR = -32603
    SERVER_ERROR = -32000

    # Application Errors
    VALIDATION_ERROR = -32001
    TOOL_ERROR = -32002
    LLM_ERROR = -32003
    SESSION_ERROR = -32004
    CONFIG_ERROR = -32005
    INTERRUPTED = -32006


# =============================================================================
# Tool Status
# =============================================================================


class ToolStatus(str, Enum):
    """Tool execution status states."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


# =============================================================================
# Agent States
# =============================================================================


class AgentState(str, Enum):
    """Agent execution states."""

    IDLE = "idle"
    THINKING = "thinking"  # LLM generating
    EXECUTING = "executing"  # Tool running
    COMPACTING = "compacting"  # History compaction
    ERROR = "error"


# =============================================================================
# Configuration Keys
# =============================================================================


class ConfigKey(str, Enum):
    """Configuration environment variable keys."""

    # Core
    MODEL = "BERG10_AGENT_MODEL"
    API_KEY = "BERG10_AGENT_API_KEY"
    BASE_URL = "BERG10_AGENT_BASE_URL"
    WORK_DIR = "BERG10_AGENT_WORK_DIR"

    # Server
    HOST = "BERG10_AGENT_HOST"
    PORT = "BERG10_AGENT_PORT"
    LOG_LEVEL = "BERG10_LOG_LEVEL"

    # Agent Behavior
    MAX_TURNS = "BERG10_AGENT_MAX_TURNS"
    MAX_HISTORY_TOKENS = "BERG10_AGENT_MAX_HISTORY_TOKENS"
    COMPACT_THRESHOLD = "BERG10_AGENT_COMPACT_THRESHOLD"
    TOOL_TIMEOUT = "BERG10_AGENT_TOOL_TIMEOUT"

    # Features
    ENABLE_VALIDATORS = "BERG10_AGENT_ENABLE_VALIDATORS"
    ENABLE_MEMORY = "BERG10_AGENT_ENABLE_MEMORY"
    ENABLE_SKILLS = "BERG10_AGENT_ENABLE_SKILLS"


# =============================================================================
# Default Values
# =============================================================================


class Defaults:
    """Default configuration values."""

    MODEL: Final[str] = "openrouter/anthropic/claude-3-5-sonnet-20250609"
    HOST: Final[str] = "0.0.0.0"
    PORT: Final[int] = 8765
    LOG_LEVEL: Final[str] = "INFO"
    MAX_TURNS: Final[int] = 50
    MAX_HISTORY_TOKENS: Final[int] = 8000
    COMPACT_THRESHOLD: Final[int] = 6000
    TOOL_TIMEOUT: Final[int] = 60
    ENABLE_VALIDATORS: Final[bool] = True
    ENABLE_MEMORY: Final[bool] = True
    ENABLE_SKILLS: Final[bool] = True
    WORK_DIR: Final[str] = "."


# =============================================================================
# File Paths
# =============================================================================


class FilePath:
    """Standard file paths."""

    MEMORY_FILE: Final[str] = ".memory.md"
    SKILLS_FILE: Final[str] = "AGENTS.md"
    ENV_FILE: Final[str] = ".env"


# =============================================================================
# Validation
# =============================================================================


class ValidationResult:
    """Result of input validation."""

    def __init__(self, valid: bool, reason: str = ""):
        self.valid = valid
        self.reason = reason

    @property
    def is_valid(self) -> bool:
        return self.valid

    def __bool__(self) -> bool:
        return self.valid

    def __repr__(self) -> str:
        status = "valid" if self.valid else f"invalid: {self.reason}"
        return f"ValidationResult({status})"


class ValidationLevel(str, Enum):
    """Validation strictness levels."""

    STRICT = "strict"
    MODERATE = "moderate"
    PERMISSIVE = "permissive"


# =============================================================================
# JSON-RPC 3.0 Stream Types
# =============================================================================


class StreamType(str, Enum):
    """Stream message types for JSON-RPC 3.0."""

    CONTENT = "content"  # Primary content stream
    PROGRESS = "progress"  # Progress updates
    LOG = "log"  # Log messages
    ERROR = "error"  # Error stream


# =============================================================================
# HTTP/WebSocket Status
# =============================================================================


WS_CLOSE_NORMAL: Final[int] = 1000
WS_CLOSE_GOING_AWAY: Final[int] = 1001
WS_CLOSE_PROTOCOL_ERROR: Final[int] = 1002
WS_CLOSE_UNSUPPORTED: Final[int] = 1003
WS_CLOSE_ABNORMAL: Final[int] = 1006
