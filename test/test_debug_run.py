import sys
import os
import uuid
import subprocess

# Load environment variables from .env file
from dotenv import load_dotenv
load_dotenv(os.path.join(os.path.dirname(__file__), ".env"))

# Set custom DeerFlow home directory BEFORE importing config
os.environ["DEER_FLOW_HOME"] = "/tmp/deer-flow"

# Add DeerFlow backend packages to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "3rdParty", "deer-flow", "backend", "packages", "harness"))

# Monkey-patch ClarificationMiddleware to not interrupt the stream
from deerflow.agents.middlewares.clarification_middleware import ClarificationMiddleware

def _passthrough_wrap(self, request, handler):
    return handler(request)

async def _passthrough_awrap(self, request, handler):
    return await handler(request)

ClarificationMiddleware.wrap_tool_call = _passthrough_wrap
ClarificationMiddleware.awrap_tool_call = _passthrough_awrap
print("[patched] ClarificationMiddleware disabled")

from deerflow.config.app_config import AppConfig, set_app_config
from deerflow.config.model_config import ModelConfig
from deerflow.config.sandbox_config import SandboxConfig
from deerflow.client import DeerFlowClient
from deerflow.config.paths import get_paths

API_KEY = os.environ.get("OPENCODE_API_KEY", "your-api-key-here")
MODEL_NAME = "glm-5"
BASE_URL = "https://opencode.ai/zen/v1"

IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp"}
SANDBOX_TOOLS = {"bash", "write_file", "read_file", "ls", "str_replace"}
MAX_TURNS = 6  # max auto-clarification follow-ups

config = AppConfig(
    models=[
        ModelConfig(
            name=MODEL_NAME,
            display_name=MODEL_NAME,
            use="langchain_openai:ChatOpenAI",
            model=MODEL_NAME,
            api_key=API_KEY,
            base_url=BASE_URL,
        )
    ],
    sandbox=SandboxConfig(use="src.sandbox.local:LocalSandboxProvider"),
)

set_app_config(config)


def _truncate(text, max_len=500):
    if not text:
        return ""
    if len(text) <= max_len:
        return text
    return text[:max_len] + "..."


def _format_tool_args(name, args):
    if name == "bash":
        desc = args.get("description", "")
        cmd = args.get("command", "")
        return f"desc={desc!r}, command={cmd!r}"
    if name == "present_files":
        return f"filepaths={args.get('filepaths', [])}"
    if name == "write_file":
        path = args.get("path", "")
        content = args.get("content", "")
        return f"path={path!r}, content={content!r}"
    if name == "read_file":
        return f"path={args.get('path', '')!r}"
    if name == "str_replace":
        return f"path={args.get('path', '')!r}"
    return ", ".join(f"{k}={v!r}" for k, v in args.items())


def run_stream(client, message, thread_id, turn_num):
    """Run a single stream and return (clarification_question, clarification_options)."""
    tool_call_counts = {}
    tool_result_counts = {}
    clarification_question = None
    clarification_options = None

    print(f"\n{'='*60}")
    print(f"TURN {turn_num}: {message}")
    print('='*60)

    for event in client.stream(message, thread_id=thread_id):
        if event.type == "messages-tuple":
            data = event.data
            msg_type = data.get("type")

            if msg_type == "ai":
                tool_calls = data.get("tool_calls")
                if tool_calls:
                    for tc in tool_calls:
                        name = tc.get("name", "?")
                        args = tc.get("args", {})
                        tool_call_counts[name] = tool_call_counts.get(name, 0) + 1
                        print(f"  🔧 {name}({_format_tool_args(name, args)})")

                        # Track ask_clarification
                        if name == "ask_clarification":
                            clarification_question = args.get("question", "")
                            clarification_options = args.get("options")

                content = data.get("content", "")
                if content:
                    print(f"  AI: {content}")

            elif msg_type == "tool":
                name = data.get("name", "")
                content = data.get("content", "")
                tool_result_counts[name] = tool_result_counts.get(name, 0) + 1
                if name in SANDBOX_TOOLS:
                    truncated = _truncate(content)
                    marker = "✗" if content.lower().startswith("error") else "←"
                    print(f"  {marker} {name}: {truncated}")
                else:
                    truncated = _truncate(content, max_len=300)
                    if truncated:
                        print(f"  ← {name}: {truncated}")

        elif event.type == "values":
            artifacts = event.data.get("artifacts", [])
            for artifact_vpath in artifacts:
                try:
                    host_path = get_paths().resolve_virtual_path(thread_id, artifact_vpath)
                    if host_path.exists():
                        suffix = host_path.suffix.lower()
                        if suffix in IMAGE_EXTS:
                            subprocess.run(["open", str(host_path)], check=False)
                        print(f"  📎 {host_path.name} → {host_path}")
                    else:
                        print(f"  ⚠ Artifact not found on disk: {host_path}")
                except Exception as e:
                    print(f"  ⚠ Could not resolve artifact {artifact_vpath}: {e}")

        elif event.type == "end":
            usage = event.data.get("usage", {})
            total = usage.get("total_tokens", 0)
            if total:
                print(f"  [{usage.get('input_tokens', 0)} in / {usage.get('output_tokens', 0)} out / {total} total tokens]")

    # Print tool summary
    for name, count in sorted(tool_call_counts.items()):
        if count > 2:
            print(f"  ⚠ {name} called {count} times — possible loop")
    if tool_call_counts:
        parts = []
        for name in sorted(tool_call_counts):
            calls = tool_call_counts[name]
            results = tool_result_counts.get(name, 0)
            parts.append(f"{name}({calls}c/{results}r)")
        print(f"  ── {' | '.join(parts)} ──")

    return clarification_question, clarification_options


def main():
    client = DeerFlowClient(model_name=MODEL_NAME)
    thread_id = str(uuid.uuid4())

    print(f"DeerFlow Test (model: {MODEL_NAME})")
    print(f"Thread: {thread_id}")

    # Initial prompt
    message = "Draw an image of a golden monkey in misty mountains"
    turn = 0

    while turn < MAX_TURNS:
        turn += 1
        question, options = run_stream(client, message, thread_id, turn)

        if question:
            print(f"\n  ❓ Clarification asked: {question}")
            if options:
                for i, opt in enumerate(options, 1):
                    print(f"    {i}. {opt}")
            # Auto-respond to clarification
            if options:
                message = options[0]  # pick first option
            else:
                message = "Yes, proceed with the task"
            print(f"  → Auto-responding: {message}")
        else:
            print("\n  No clarification needed — turn complete")
            break

    print(f"\n{'='*60}")
    print("DONE")
    print('='*60)


if __name__ == "__main__":
    main()
