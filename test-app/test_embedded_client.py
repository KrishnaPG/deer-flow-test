import sys
import os
import asyncio
import uuid

# Set custom DeerFlow home directory BEFORE importing config
os.environ["DEER_FLOW_HOME"] = "/tmp/deer-flow"

# Add DeerFlow backend to path (NOT src, since internal imports use src.*)
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "deer-flow", "backend"))

from src.config.app_config import AppConfig, set_app_config
from src.config.model_config import ModelConfig
from src.config.sandbox_config import SandboxConfig
from src.client import DeerFlowClient
from src.agents.memory.queue import get_memory_queue
from langchain_core.messages import HumanMessage
from langchain_core.runnables import RunnableConfig

# Hardcoded - replace with your values
API_KEY = "sk-28TxdahCE"
MODEL_NAME = "minimax-m2.5"
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
            "thinking_enabled": True,
        },
        recursion_limit=100,
    )
    
    # Ensure agent is created
    client._ensure_agent(config)
    
    state = {"messages": [HumanMessage(content="What is 2 + 2?")]}
    context = {"thread_id": thread_id}
    
    # Use async stream - print in real-time
    print("Response: ", end="", flush=True)
    seen_ids = set()
    async for chunk in client._agent.astream(state, config=config, context=context, stream_mode="values"):
        messages = chunk.get("messages", [])
        for msg in messages:
            msg_id = getattr(msg, "id", None)
            if msg_id and msg_id in seen_ids:
                continue
            if msg_id:
                seen_ids.add(msg_id)
            
            # Print AI message content as it arrives
            if hasattr(msg, "type") and msg.type == "ai":
                content = getattr(msg, "content", "")
                if content:
                    print(content, end="", flush=True)
    
    print()  # newline at end
    
    # Force memory write (normally has 30s delay)
    print("Flushing memory...")
    get_memory_queue().flush()
    print("Memory flush complete")

if __name__ == "__main__":
    asyncio.run(main())
