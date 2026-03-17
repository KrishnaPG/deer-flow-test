import sys
import os
import asyncio
import uuid

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
from deerflow.agents.memory.queue import get_memory_queue
from langchain_core.messages import HumanMessage
from langchain_core.runnables import RunnableConfig

# Hardcoded - replace with your values
API_KEY = os.environ.get("API_KEY", "your-api-key-here")
MODEL_NAME = "nemotron-3-super-free"
BASE_URL = "https://opencode.ai/zen/v1"  # Optional, defaults to OpenAI

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

async def main():
    client = DeerFlowClient(model_name=MODEL_NAME)
    
    thread_id = str(uuid.uuid4())
    config = RunnableConfig(
        configurable={
            "thread_id": thread_id,
            "model_name": MODEL_NAME,
            "thinking_enabled": False,
        },
        recursion_limit=100,
    )
    
    # Ensure agent is created
    client._ensure_agent(config)
    
    state = {"messages": [HumanMessage(content="Draw an image of Deer in a forest")]}
    context = {"thread_id": thread_id}
    
    # Use async stream - print all messages
    print("\n=== Messages from Agent ===")
    seen_ids = set()
    async for chunk in client._agent.astream(state, config=config, context=context, stream_mode="values"):
        messages = chunk.get("messages", [])
        for msg in messages:
            msg_id = getattr(msg, "id", None)
            if msg_id and msg_id in seen_ids:
                continue
            if msg_id:
                seen_ids.add(msg_id)
            
            # Print all message types
            msg_type = getattr(msg, "type", "unknown")
            content = getattr(msg, "content", "")
            name = getattr(msg, "name", None)
            
            print(f"\n[{msg_type.upper()}]", end="")
            if name:
                print(f" {name}", end="")
            print(":")
            if content:
                print(content)
            else:
                print("(no content)")
    
    print("\n=== End of Messages ===\n")
    
    # Cancel memory queue timer to avoid 30s wait
    memory_queue = get_memory_queue()
    if memory_queue._timer:
        memory_queue._timer.cancel()
        memory_queue._timer = None

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)
