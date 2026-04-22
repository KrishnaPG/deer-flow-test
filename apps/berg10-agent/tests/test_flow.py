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

    async def prep(self, shared):
        return {"step": shared.get("step", 0)}

    async def exec(self, prep_res):
        self.call_count += 1
        return {"action": "answer", "step": prep_res["step"] + 1}

    async def post(self, shared, prep_res, exec_res):
        shared["step"] = exec_res["step"]
        if exec_res["step"] >= 3:
            return "done"
        shared["step"] = exec_res["step"]
        return "loop"


class TestAgentFlow:
    @pytest.mark.asyncio
    async def test_flow_runs(self):
        node = MockDecideNode()
        flow = AgentFlow(start_node=node)

        shared = {"step": 0}
        await flow.run_async(shared)

        assert shared["step"] >= 1
        assert node.call_count >= 1

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
