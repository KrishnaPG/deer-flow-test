import sys
import os
import uuid
import subprocess
import json
import hashlib
from collections import defaultdict, Counter

from dotenv import load_dotenv
load_dotenv(os.path.join(os.path.dirname(__file__), ".env"))

os.environ["DEER_FLOW_HOME"] = "/tmp/deer-flow"

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "3rdParty", "deer-flow", "backend", "packages", "harness"))

from deerflow.config.app_config import AppConfig, set_app_config
from deerflow.config.model_config import ModelConfig
from deerflow.config.sandbox_config import SandboxConfig
from deerflow.client import DeerFlowClient
from deerflow.config.paths import get_paths

# Colorized console output (optional dependency: colorama)
try:
    from colorama import Fore, Style, init as _colorama_init
except Exception:
    Fore = None
    Style = None
    _colorama_init = None

if _colorama_init:
    _colorama_init(autoreset=True)

def colored(text: str, color: str = None, bright: bool = True) -> str:
    if not Fore or not Style or color is None:
        return text
    color_code = getattr(Fore, color.upper(), "")
    bright_code = Style.BRIGHT if bright else ""
    return f"{bright_code}{color_code}{text}{Style.RESET_ALL}"

def print_block(header: str, body: str | None = None, color: str = "CYAN"):
    print(colored(f"\n--- {header} ---", color))
    if body:
        print(body)
    print(colored(f"--- end {header} ---\n", color, bright=False))

API_KEY = os.environ.get("OPENCODE_API_KEY", "your-api-key-here")
MODEL_NAME = "big-pickle"
BASE_URL = "https://opencode.ai/zen/v1"

IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp"}
SANDBOX_TOOLS = {"bash", "write_file", "read_file", "ls", "str_replace"}

# Track repeated present_files calls per-thread for debugging / loop detection
PRESENT_FILES_HISTORY: dict[str, Counter] = defaultdict(Counter)
PRESENT_FILES_REPEAT_LIMIT = 3

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
        return f"desc={args.get('description', '')!r}, command={args.get('command', '')!r}"
    if name == "present_files":
        return f"filepaths={args.get('filepaths', [])}"
    if name == "write_file":
        return f"path={args.get('path', '')!r}, content={args.get('content', '')!r}"
    if name == "read_file":
        return f"path={args.get('path', '')!r}"
    if name == "str_replace":
        return f"path={args.get('path', '')!r}"
    if name == "ask_clarification":
        return f"question={args.get('question', '')!r}, options={args.get('options')!r}"
    return ", ".join(f"{k}={v!r}" for k, v in args.items())


def process_stream(client, message, thread_id, seen_msg_ids):
    """Stream events for a message. Returns (question, options) if clarification was asked."""
    tool_call_counts = {}
    tool_result_counts = {}
    clarification_question = None
    clarification_options = None

    print()
    for event in client.stream(message, thread_id=thread_id):
        # Raw dumps for debugging: messages-tuple and values
        try:
            if event.type == "messages-tuple":
                print_block("RAW MESSAGE-TUPLE", json.dumps(event.data, indent=2, default=str), color="YELLOW")
            if event.type == "values":
                print_block("RAW VALUES SNAPSHOT", json.dumps(event.data, indent=2, default=str), color="CYAN")
        except Exception:
            pass
        if event.type == "messages-tuple":
            data = event.data
            msg_id = data.get("id")
            if msg_id and msg_id in seen_msg_ids:
                continue
            if msg_id:
                seen_msg_ids.add(msg_id)

            msg_type = data.get("type")

            if msg_type == "ai":
                tool_calls = data.get("tool_calls")
                if tool_calls:
                    for tc in tool_calls:
                        name = tc.get("name", "?")
                        args = tc.get("args", {})
                        tool_call_counts[name] = tool_call_counts.get(name, 0) + 1
                        tool_call_id = tc.get("id")
                        print(colored(f"  \U0001f527 {name}({_format_tool_args(name, args)}) [id={tool_call_id}]", "BLUE"))

                        if name == "ask_clarification":
                            clarification_question = args.get("question", "")
                            clarification_options = args.get("options")
                            # also print the formatted clarification block
                            print_block("CLARIFICATION (tool_calls)", clarification_question or "", color="BLUE")
                            if clarification_options:
                                for i, opt in enumerate(clarification_options, 1):
                                    print(colored(f"    {i}. {opt}", "BLUE"))

                        if name == "present_files":
                            # Handle present_files intents immediately for debugging
                            filepaths = args.get("filepaths") or []
                            # Print vpaths
                            print_block("PRESENT_FILES (intent)", json.dumps(filepaths, indent=2, default=str), color="YELLOW")

                            # Hash the set of filepaths to detect repeats
                            norm = json.dumps(sorted(filepaths), separators=(",", ":"), ensure_ascii=False)
                            h = hashlib.md5(norm.encode()).hexdigest()[:12]
                            PRESENT_FILES_HISTORY[thread_id][h] += 1
                            count = PRESENT_FILES_HISTORY[thread_id][h]
                            if count > PRESENT_FILES_REPEAT_LIMIT:
                                print(colored(f"\n[WARNING] present_files repeated {count} times for this thread — suppressing auto-open.\n", "YELLOW"))

                            # Attempt to resolve and open any referenced paths now
                            for vpath in filepaths:
                                try:
                                    host_path = get_paths().resolve_virtual_path(thread_id, vpath)
                                except Exception as e:
                                    print(colored(f"  ⚠ Could not resolve vpath {vpath}: {e}", "RED"))
                                    host_path = None

                                if host_path:
                                    try:
                                        exists = host_path.exists()
                                    except Exception:
                                        exists = False
                                    if exists:
                                        suffix = host_path.suffix.lower()
                                        print(colored(f"  📎 Resolved: {vpath} -> {host_path}", "CYAN"))
                                        # Auto-open images unless we've suppressed due to repeats
                                        if count <= PRESENT_FILES_REPEAT_LIMIT and suffix in IMAGE_EXTS:
                                            try:
                                                subprocess.run(["open", str(host_path)], check=False)
                                            except Exception as e:
                                                print(colored(f"  ⚠ Failed to open file {host_path}: {e}", "RED"))
                                    else:
                                        # Poll briefly to allow delayed writes
                                        found = False
                                        for attempt in range(6):
                                            try:
                                                if host_path.exists():
                                                    found = True
                                                    break
                                            except Exception:
                                                pass
                                            print(colored(f"  …waiting for artifact to appear ({attempt+1}/6)", "YELLOW"))
                                            import time
                                            time.sleep(0.5)

                                        if found:
                                            print(colored(f"  📎 Artifact appeared: {host_path}", "GREEN"))
                                            suffix = host_path.suffix.lower()
                                            if count <= PRESENT_FILES_REPEAT_LIMIT and suffix in IMAGE_EXTS:
                                                try:
                                                    subprocess.run(["open", str(host_path)], check=False)
                                                except Exception as e:
                                                    print(colored(f"  ⚠ Failed to open file {host_path}: {e}", "RED"))
                                        else:
                                            print(colored(f"  ⚠ Artifact not found on disk: {host_path}", "YELLOW"))

                content = data.get("content", "")
                if content:
                    print(colored(f"  AI: {content}", "MAGENTA"))

            elif msg_type == "tool":
                name = data.get("name", "")
                if name == "ask_clarification":
                    continue
                content = data.get("content", "")
                tool_result_counts[name] = tool_result_counts.get(name, 0) + 1
                if name in SANDBOX_TOOLS:
                    truncated = _truncate(content, max_len=500)
                    if content.lower().startswith("error"):
                        print(colored(f"  \u2717 {name}: {truncated}", "RED"))
                    else:
                        print(colored(f"  \u2190 {name}: {truncated}", "GREEN"))
                else:
                    truncated = _truncate(content)
                    if truncated:
                        print(colored(f"  \u2190 {name}: {truncated}", "CYAN"))

        elif event.type == "values":
            artifacts = event.data.get("artifacts", [])
            for artifact_vpath in artifacts:
                try:
                    host_path = get_paths().resolve_virtual_path(thread_id, artifact_vpath)
                    if host_path.exists():
                        suffix = host_path.suffix.lower()
                        if suffix in IMAGE_EXTS:
                            subprocess.run(["open", str(host_path)], check=False)
                        print(f"  \U0001f4ce {host_path.name} \u2192 {host_path}")
                    else:
                        print(f"  \u26a0 Artifact not found on disk: {host_path}")
                except Exception as e:
                    print(f"  \u26a0 Could not resolve artifact {artifact_vpath}: {e}")

        elif event.type == "end":
            usage = event.data.get("usage", {})
            total = usage.get("total_tokens", 0)
            if total:
                print(f"\n  [{usage.get('input_tokens', 0)} in / {usage.get('output_tokens', 0)} out / {total} total tokens]")

    for name, count in sorted(tool_call_counts.items()):
        if count > 2:
            print(f"  \u26a0 {name} called {count} times \u2014 possible loop")
    if tool_call_counts:
        parts = [f"{n}({tool_call_counts[n]}c/{tool_result_counts.get(n, 0)}r)" for n in sorted(tool_call_counts)]
        print(f"  \u2500\u2500 {' | '.join(parts)} \u2500\u2500")

    return clarification_question, clarification_options


def main():
    client = DeerFlowClient(model_name=MODEL_NAME)
    # Print system prompt used by the agent (best-effort)
    try:
        from deerflow.agents.lead_agent.prompt import apply_prompt_template
        system_prompt = apply_prompt_template(subagent_enabled=False, max_concurrent_subagents=3, agent_name=None)
        print_block("SYSTEM PROMPT", system_prompt, color="MAGENTA")
    except Exception:
        # best-effort: ignore if prompt cannot be rendered in this environment
        pass
    thread_id = str(uuid.uuid4())
    seen_msg_ids = set()

    print(f"DeerFlow Interactive Chat (model: {MODEL_NAME})")
    print(f"Thread: {thread_id}")
    print("Commands: /new (new conversation), /quit")
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
            print(f"(New conversation started \u2014 thread: {thread_id})")
            continue

        try:
            while True:
                question, options = process_stream(
                    client, user_input, thread_id, seen_msg_ids
                )
                if not question:
                    break
                print(f"\n  \u2753 {question}")
                if options:
                    for i, opt in enumerate(options, 1):
                        print(f"    {i}. {opt}")
                try:
                    user_input = input("\nYou: ").strip()
                except (EOFError, KeyboardInterrupt):
                    print("\nBye!")
                    return
                if not user_input:
                    break
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    main()
