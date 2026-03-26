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

from deerflow.config.app_config import AppConfig, set_app_config
from deerflow.config.model_config import ModelConfig
from deerflow.config.sandbox_config import SandboxConfig
from deerflow.client import DeerFlowClient
from deerflow.config.paths import get_paths

# Hardcoded - replace with your values
API_KEY = os.environ.get("OPENCODE_API_KEY", "your-api-key-here")
MODEL_NAME = "nemotron-3-super-free"
BASE_URL = "https://opencode.ai/zen/v1"

IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp"}

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


def handle_event(event, thread_id):
    """Process a single stream event and display relevant output."""
    if event.type == "messages-tuple":
        data = event.data
        msg_type = data.get("type")

        if msg_type == "ai":
            # Show tool calls
            tool_calls = data.get("tool_calls")
            if tool_calls:
                for tc in tool_calls:
                    name = tc.get("name", "?")
                    args = tc.get("args", {})
                    if name == "present_files":
                        fps = args.get("filepaths", [])
                        print(f"  🔧 {name}(filepaths={fps})")
                    else:
                        print(f"  🔧 {name}({', '.join(f'{k}={v!r}' for k, v in args.items())})")

            # Show AI text content
            content = data.get("content", "")
            if content:
                print(content)

    elif event.type == "values":
        # Detect artifacts and auto-open
        artifacts = event.data.get("artifacts", [])
        for artifact_vpath in artifacts:
            try:
                host_path = get_paths().resolve_virtual_path(thread_id, artifact_vpath)
                if host_path.exists():
                    suffix = host_path.suffix.lower()
                    if suffix in IMAGE_EXTS:
                        subprocess.run(["open", str(host_path)], check=False)
                    print(f"  📎 {host_path.name} → {host_path}")
            except Exception as e:
                print(f"  ⚠ Could not resolve artifact {artifact_vpath}: {e}")

    elif event.type == "end":
        usage = event.data.get("usage", {})
        total = usage.get("total_tokens", 0)
        if total:
            print(f"\n  [{usage.get('input_tokens', 0)} in / {usage.get('output_tokens', 0)} out / {total} total tokens]")


def main():
    client = DeerFlowClient(model_name=MODEL_NAME)
    thread_id = str(uuid.uuid4())

    print(f"DeerFlow Interactive Chat (model: {MODEL_NAME})")
    print(f"Thread: {thread_id}")
    print("Type your message, /new for a new conversation, /quit to exit.")
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
            print(f"(New conversation started — thread: {thread_id})")
            continue

        print()
        try:
            for event in client.stream(user_input, thread_id=thread_id):
                handle_event(event, thread_id)
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    main()
