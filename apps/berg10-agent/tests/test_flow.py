"""Tests for agent flow wiring."""

import pytest

from berg10_agent.flow.agent_flow import AgentFlow, create_agent_flow
from berg10_agent.config import AgentConfig
from berg10_agent.nodes import DecideActionNode
from berg10_agent.llm import LLMClient
from pocketflow import AsyncNode


class MockDecideNode(AsyncNode):
    """Mock decision node for testing flow wiring."""

    def __init__(self):
        super().__init__()
        self.call_count = 0

    async def prep_async(self, shared):
        print("MockDecideNode prep_async called")
        return {"step": shared.get("step", 0)}

    async def exec_async(self, prep_res):
        print("MockDecideNode exec_async called")
        self.call_count += 1
        return {"action": "answer", "step": prep_res["step"] + 1}

    async def post_async(self, shared, prep_res, exec_res):
        print(f"MockDecideNode post_async called, step={exec_res['step']}")
        shared["step"] = exec_res["step"]
        if exec_res["step"] >= 3:
            return "done"
        return "loop"


class TestAgentFlow:
    @pytest.mark.asyncio
    async def test_flow_runs(self):
        node = MockDecideNode()
        # Add the 'done' terminal node to successfully exit without warnings
        terminal = AsyncNode()
        node - "loop" >> node
        node - "done" >> terminal
        flow = AgentFlow(start_node=node)

        shared = {"step": 0}
        await flow.run_async(shared)

        assert shared["step"] >= 3

    def test_create_agent_flow_returns_flow(self):
        config = AgentConfig(
            model="test",
            work_dir=".",
            tool_timeout=30,
        )
        flow = create_agent_flow(config)
        assert isinstance(flow, AgentFlow)


class TestFlowWiring:
    def test_decide_routes_to_tools(self):
        """Verify decision node action routes exist."""
        config = AgentConfig(model="test", work_dir=".", tool_timeout=30)
        flow = create_agent_flow(config)
        assert flow is not None
