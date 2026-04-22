"""Agent flow wiring using PocketFlow AsyncFlow."""

from __future__ import annotations

from pocketflow import AsyncFlow, AsyncNode

from ..config import AgentConfig
from ..llm import LLMClient
from ..nodes import (
    CompactHistoryNode,
    DecideActionNode,
    DoneNode,
    GrepSearchNode,
    ListFilesNode,
    PatchFileNode,
    ReadFileNode,
    RunCommandNode,
)


class AgentFlow(AsyncFlow):
    """Main agent flow: decide action -> execute tool -> repeat."""

    def __init__(self, start_node: AsyncNode) -> None:
        super().__init__(start=start_node)


def create_agent_flow(config: AgentConfig) -> AgentFlow:
    """Create and wire the agent flow graph."""
    llm = LLMClient(
        model=config.model,
        api_key=config.api_key,
        base_url=config.base_url,
    )

    # Create nodes
    decide = DecideActionNode(llm)
    done = DoneNode()
    compact = CompactHistoryNode(
        llm, max_tokens=config.max_history_tokens, compact_threshold=config.compact_threshold
    )
    list_files = ListFilesNode(work_dir=config.work_dir)
    grep_search = GrepSearchNode(work_dir=config.work_dir)
    read_file = ReadFileNode(work_dir=config.work_dir)
    patch_file = PatchFileNode(work_dir=config.work_dir)
    run_command = RunCommandNode(work_dir=config.work_dir, default_timeout=config.tool_timeout)

    # Wire: decide -> tool -> decide (cyclic via post routing)
    # Each tool node routes back to "decide" in its post method
    decide - "answer" >> done
    decide - "tool_list_files" >> list_files
    decide - "tool_grep_search" >> grep_search
    decide - "tool_read_file" >> read_file
    decide - "tool_patch_file" >> patch_file
    decide - "tool_run_command" >> run_command

    # Compact hooks into the decision path
    decide - "compact" >> compact
    compact - "decide" >> decide

    return AgentFlow(start_node=decide)
