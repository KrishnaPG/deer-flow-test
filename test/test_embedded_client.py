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

# Monkey-patch ClarificationMiddleware BEFORE importing DeerFlow client
# This makes ask_clarification work as a regular tool (no Command(goto=END))
from deerflow.agents.middlewares.clarification_middleware import ClarificationMiddleware

def _passthrough_wrap_tool_call(self, request, handler):
    return handler(request)

async def _passthrough_awrap_tool_call(self, request, handler):
    return await handler(request)

ClarificationMiddleware.wrap_tool_call = _passthrough_wrap_tool_call
ClarificationMiddleware.awrap_tool_call = _passthrough_awrap_tool_call
print("[patched] ClarificationMiddleware disabled — ask_clarification will work as a regular tool")

from deerflow.config.app_config import AppConfig, set_app_config
from deerflow.config.model_config import ModelConfig
from deerflow.config.sandbox_config import SandboxConfig
from deerflow.client import DeerFlowClient
from deerflow.config.paths import get_paths

# Hardcoded - replace with your values
API_KEY = os.environ.get("OPENCODE_API_KEY", "your-api-key-here")
MODEL_NAME = "glm-5"
BASE_URL = "https://opencode.ai/zen/v1"

IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp"}
SANDBOX_TOOLS = {"bash", "write_file", "read_file", "ls", "str_replace"}

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


def _truncate(text, max_len=200):
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
    if name == "ask_clarification":
        q = args.get("question", "")
        opts = args.get("options")
        return f"question={q!r}, options={opts!r}"
    return ", ".join(f"{k}={v!r}" for k, v in args.items())


def handle_event(event, thread_id, tool_call_counts, tool_result_counts, verbose,
                 seen_msg_ids=None, clarification_tracker=None):
    """Process a single stream event and display relevant output."""
    # Dedup messages across turns
    if event.type == "messages-tuple" and seen_msg_ids is not None:
        msg_id = event.data.get("id")
        if msg_id and msg_id in seen_msg_ids:
            return
        if msg_id:
            seen_msg_ids.add(msg_id)

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
                    if name == "ask_clarification" and clarification_tracker is not None:
                        clarification_tracker["question"] = args.get("question", "")
                        clarification_tracker["options"] = args.get("options")

            content = data.get("content", "")
            if content:
                print(f"  AI: {content}")

        elif msg_type == "tool":
            name = data.get("name", "")
            content = data.get("content", "")
            tool_result_counts[name] = tool_result_counts.get(name, 0) + 1
            if name in SANDBOX_TOOLS:
                truncated = _truncate(content, max_len=500)
                marker = "✗" if content.lower().startswith("error") else "←"
                if truncated:
                    print(f"  {marker} {name}: {truncated}")
                else:
                    print(f"  {marker} {name}: (empty)")
            elif verbose:
                truncated = _truncate(content)
                if truncated:
                    print(f"  ← {name}: {truncated}")
                else:
                    print(f"  ← {name}: (empty)")

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
            print(f"\n  [{usage.get('input_tokens', 0)} in / {usage.get('output_tokens', 0)} out / {total} total tokens]")


def print_tool_summary(tool_call_counts, tool_result_counts):
    for name, count in sorted(tool_call_counts.items()):
        if count > 2:
            print(f"  ⚠ {name} called {count} times — possible loop")
    if not tool_call_counts:
        return
    parts = []
    for name in sorted(tool_call_counts):
        calls = tool_call_counts[name]
        results = tool_result_counts.get(name, 0)
        parts.append(f"{name}({calls}c/{results}r)")
    print(f"  ── {' | '.join(parts)} ──")


def run_turn(client, message, thread_id, seen_msg_ids, verbose):
    """Run a single stream turn. Returns True if a clarification was asked."""
    tool_call_counts = {}
    tool_result_counts = {}
    clarification = {"question": None, "options": None}

    print()
    for event in client.stream(message, thread_id=thread_id):
        handle_event(event, thread_id, tool_call_counts, tool_result_counts, verbose,
                     seen_msg_ids=seen_msg_ids, clarification_tracker=clarification)

    print_tool_summary(tool_call_counts, tool_result_counts)

    if clarification["question"]:
        print(f"\n  ❓ {clarification['question']}")
        if clarification["options"]:
            for i, opt in enumerate(clarification["options"], 1):
                print(f"    {i}. {opt}")
        return True
    return False


def main():
    client = DeerFlowClient(model_name=MODEL_NAME)
    thread_id = str(uuid.uuid4())
    verbose = True
    seen_msg_ids = set()  # global dedup across turns

    print(f"DeerFlow Interactive Chat (model: {MODEL_NAME})")
    print(f"Thread: {thread_id}")
    print("Commands: /new (new conversation), /verbose (toggle tool logging), /quit")
    print("-" * 60)

    while True:
        try:
            user_input = input("\nYou: ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\nBye!")
            break

        if not user_input:
            continue

        cmd = user_input.lower()
        if cmd in ("/quit", "/exit", "/q"):
            print("Bye!")
            break
        if cmd == "/new":
            thread_id = str(uuid.uuid4())
            seen_msg_ids.clear()
            print(f"(New conversation started — thread: {thread_id})")
            continue
        if cmd == "/verbose":
            verbose = not verbose
            print(f"(Verbose mode: {'on' if verbose else 'off'})")
            continue

        # Run turn — if clarification is asked, show it and loop for more input
        try:
            asked = run_turn(client, user_input, thread_id, seen_msg_ids, verbose)
            if asked:
                # Show clarification, let user respond naturally on next input
                # The agent already received the ask_clarification tool result
                # and can proceed with other tool calls on the next turn
                pass
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    main()
