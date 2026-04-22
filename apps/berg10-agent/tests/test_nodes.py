"""Tests for tool nodes."""

import pytest
import asyncio
import tempfile
import os
from pathlib import Path

from berg10_agent.nodes.list_files import ListFilesNode
from berg10_agent.nodes.grep_search import GrepSearchNode
from berg10_agent.nodes.read_file import ReadFileNode
from berg10_agent.nodes.run_command import RunCommandNode


@pytest.fixture
def work_dir():
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test files
        (Path(tmpdir) / "test.txt").write_text("hello world\nfoo bar\nbaz qux\n")
        (Path(tmpdir) / "sub").mkdir()
        (Path(tmpdir) / "sub" / "nested.py").write_text("def foo():\n    return 42\n")
        yield tmpdir


def _make_shared(tool_name: str, args: dict) -> dict:
    """Create a shared dict with a mock tool call."""
    return {
        "current_tool_call": type(
            "TC",
            (),
            {
                "id": "tc-1",
                "name": tool_name,
                "arguments": args,
            },
        )(),
        "history": [],
    }


class TestListFilesNode:
    @pytest.mark.asyncio
    async def test_list_root(self, work_dir):
        node = ListFilesNode(work_dir=work_dir)
        shared = _make_shared("list_files", {"path": ".", "pattern": "*"})

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert "entries" in result
        names = [e["name"] for e in result["entries"]]
        assert "test.txt" in names
        assert "sub" in names

    @pytest.mark.asyncio
    async def test_list_subdirectory(self, work_dir):
        node = ListFilesNode(work_dir=work_dir)
        shared = _make_shared("list_files", {"path": "sub", "pattern": "*"})

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert result["entries"][0]["name"] == "nested.py"

    @pytest.mark.asyncio
    async def test_path_traversal_blocked(self, work_dir):
        node = ListFilesNode(work_dir=work_dir)
        shared = _make_shared("list_files", {"path": "../../../etc", "pattern": "*"})

        prep = await node.prep(shared)
        result = await node.exec(prep)
        assert "error" in result


class TestGrepSearchNode:
    @pytest.mark.asyncio
    async def test_find_pattern(self, work_dir):
        node = GrepSearchNode(work_dir=work_dir)
        shared = _make_shared("grep_search", {"pattern": "foo", "path": ".", "glob": "*"})

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert "results" in result
        assert any(r["content"].find("foo") >= 0 for r in result["results"])

    @pytest.mark.asyncio
    async def test_no_match(self, work_dir):
        node = GrepSearchNode(work_dir=work_dir)
        shared = _make_shared(
            "grep_search", {"pattern": "xyznonexistent", "path": ".", "glob": "*"}
        )

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert result["results"] == []

    @pytest.mark.asyncio
    async def test_invalid_regex(self, work_dir):
        node = GrepSearchNode(work_dir=work_dir)
        shared = _make_shared("grep_search", {"pattern": "[invalid", "path": ".", "glob": "*"})

        prep = await node.prep(shared)
        result = await node.exec(prep)
        assert "error" in result


class TestReadFileNode:
    @pytest.mark.asyncio
    async def test_read_full_file(self, work_dir):
        node = ReadFileNode(work_dir=work_dir)
        shared = _make_shared("read_file", {"path": "test.txt"})

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert result["content"] == "hello world\nfoo bar\nbaz qux\n"
        assert result["total_lines"] == 3

    @pytest.mark.asyncio
    async def test_read_line_range(self, work_dir):
        node = ReadFileNode(work_dir=work_dir)
        shared = _make_shared("read_file", {"path": "test.txt", "start_line": 2, "end_line": 2})

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert result["content"] == "foo bar"

    @pytest.mark.asyncio
    async def test_read_nonexistent(self, work_dir):
        node = ReadFileNode(work_dir=work_dir)
        shared = _make_shared("read_file", {"path": "nonexistent.txt"})

        prep = await node.prep(shared)
        result = await node.exec(prep)
        assert "error" in result


class TestRunCommandNode:
    @pytest.mark.asyncio
    async def test_simple_echo(self, work_dir):
        node = RunCommandNode(work_dir=work_dir, default_timeout=10)
        shared = _make_shared("run_command", {"command": "echo hello"})

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert result["success"] is True
        assert "hello" in result["stdout"]
        assert result["exit_code"] == 0

    @pytest.mark.asyncio
    async def test_failing_command(self, work_dir):
        node = RunCommandNode(work_dir=work_dir, default_timeout=10)
        shared = _make_shared("run_command", {"command": "exit 1"})

        prep = await node.prep(shared)
        result = await node.exec(prep)

        assert result["exit_code"] == 1
