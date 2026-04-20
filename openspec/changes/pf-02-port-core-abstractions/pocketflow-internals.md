# PocketFlow Internal Mechanics Reference

This document captures the internal mechanics of Python PocketFlow for reference when implementing the Rust+Dapr port. It is a direct transcription of observed behavior, not reinterpretation.

---

## Class Hierarchy

```
BaseNode (base class with successors routing)
├── Node (sync node with retry/fallback)
│   └── BatchNode (processes iterable items)
├── Flow (orchestrator - IS-A BaseNode)
│   └── BatchFlow (runs flow multiple times)
├── AsyncNode (async node)
│   ├── AsyncBatchNode
│   └── AsyncParallelBatchNode
└── AsyncFlow (async orchestrator - IS-A Flow AND AsyncNode)
    ├── AsyncBatchFlow
    └── AsyncParallelBatchFlow
```

**Key Insight**: `Flow` inherits from `BaseNode`, meaning a `Flow` can be used as a node in another `Flow`. This enables hierarchical composition.

---

## BaseNode Internals

```python
class BaseNode:
    def __init__(self):
        self.params = {}        # Node parameters (dict)
        self.successors = {}    # action -> Node mapping
    
    def set_params(self, params): 
        self.params = params    # Mutates self.params
    
    def next(self, node, action="default"):
        # Add edge: self.successors[action] = node
        if action in self.successors:
            warnings.warn(f"Overwriting successor for action '{action}'")
        self.successors[action] = node
        return node  # Enable chaining
    
    # Operator overloading
    def __rshift__(self, other): 
        return self.next(other)  # node >> next_node (action="default")
    
    def __sub__(self, action):
        if isinstance(action, str):
            return _ConditionalTransition(self, action)
        raise TypeError("Action must be a string")
    
    # Execution pipeline
    def _exec(self, prep_res): return self.exec(prep_res)
    
    def _run(self, shared):
        p = self.prep(shared)
        e = self._exec(p)
        return self.post(shared, p, e)  # Returns action string
    
    def run(self, shared):
        if self.successors:
            warnings.warn("Node won't run successors. Use Flow.")
        return self._run(shared)
```

### _ConditionalTransition Helper

```python
class _ConditionalTransition:
    def __init__(self, src, action):
        self.src = src
        self.action = action
    
    def __rshift__(self, tgt):
        return self.src.next(tgt, self.action)  # src - "action" >> tgt
```

**Usage**:
```python
node_a >> node_b              # sequential (action="default")
node_a - "search" >> node_b   # conditional (action="search")
```

---

## Node Internals

```python
class Node(BaseNode):
    def __init__(self, max_retries=1, wait=0):
        super().__init__()
        self.max_retries = max_retries
        self.wait = wait
    
    def exec_fallback(self, prep_res, exc):
        raise exc  # Default: re-raise exception
    
    def _exec(self, prep_res):
        for self.cur_retry in range(self.max_retries):
            try:
                return self.exec(prep_res)
            except Exception as e:
                if self.cur_retry == self.max_retries - 1:
                    return self.exec_fallback(prep_res, e)
                if self.wait > 0:
                    time.sleep(self.wait)
```

---

## BatchNode Internals

```python
class BatchNode(Node):
    def _exec(self, items):
        # items is an iterable (list from prep())
        # Calls super()._exec() once per item
        return [super(BatchNode, self)._exec(i) for i in (items or [])]
```

**Key Insight**: `BatchNode._exec()` receives an iterable `items` and calls the parent's `_exec` once per item, collecting results.

---

## Flow Internals

```python
class Flow(BaseNode):
    def __init__(self, start=None):
        super().__init__()
        self.start_node = start  # Entry point only!
    
    def start(self, start):
        self.start_node = start
        return start
    
    def get_next_node(self, curr, action):
        nxt = curr.successors.get(action or "default")
        if not nxt and curr.successors:
            warnings.warn(f"Flow ends: '{action}' not found in {list(curr.successors)}")
        return nxt
    
    def _orch(self, shared, params=None):
        curr = copy.copy(self.start_node)          # COPY start node
        p = params or {**self.params}             # Copy or use provided params
        last_action = None
        
        while curr:
            curr.set_params(p)                     # Mutate curr's params
            last_action = curr._run(shared)       # Run node, get action
            curr = copy.copy(self.get_next_node(curr, last_action))  # COPY next
        
        return last_action
    
    def _run(self, shared):
        p = self.prep(shared)
        o = self._orch(shared)
        return self.post(shared, p, o)  # Flow.post() returns exec_res directly
    
    def post(self, shared, prep_res, exec_res):
        return exec_res  # Flow doesn't transform action
```

**Key Insights**:
- `Flow` ONLY stores `start_node` - the rest of the graph is distributed in nodes' `successors`
- `copy.copy()` is used at each step to avoid mutating original node state
- `_orch` loop: `while curr: curr._run() → action → lookup next → copy → repeat`
- When `get_next_node()` returns `None`, loop terminates

### Flow as Node (Hierarchical Composition)

Because `Flow` extends `BaseNode`, it can be used as a node:

```python
agent_flow = create_agent_inner_flow()
supervisor = SupervisorNode()
agent_flow >> supervisor  # agent_flow is a node in parent graph
```

When the parent Flow reaches `agent_flow`, it calls `agent_flow._run()` which triggers its internal `_orch()`.

---

## BatchFlow Internals

```python
class BatchFlow(Flow):
    def _run(self, shared):
        pr = self.prep(shared) or []
        for bp in pr:
            self._orch(shared, {**self.params, **bp})  # Run with merged params
        return self.post(shared, pr, None)
```

**Key Insight**: `prep()` returns a list of parameter sets, and `_orch()` runs once per set with merged params.

---

## AsyncNode Internals

```python
class AsyncNode(Node):
    async def prep_async(self, shared): pass
    async def exec_async(self, prep_res): pass
    async def exec_fallback_async(self, prep_res, exc): raise exc
    async def post_async(self, shared, prep_res, exec_res): pass
    
    async def _exec(self, prep_res):
        for self.cur_retry in range(self.max_retries):
            try:
                return await self.exec_async(prep_res)
            except Exception as e:
                if self.cur_retry == self.max_retries - 1:
                    return await self.exec_fallback_async(prep_res, e)
                if self.wait > 0:
                    await asyncio.sleep(self.wait)
    
    async def run_async(self, shared):
        if self.successors:
            warnings.warn("Node won't run successors. Use AsyncFlow.")
        return await self._run_async(shared)
    
    async def _run_async(self, shared):
        p = await self.prep_async(shared)
        e = await self._exec(p)
        return await self.post_async(shared, p, e)
    
    def _run(self, shared):
        raise RuntimeError("Use run_async.")  # Sync run forbidden
```

---

## AsyncFlow Internals

```python
class AsyncFlow(Flow, AsyncNode):
    async def _orch_async(self, shared, params=None):
        curr = copy.copy(self.start_node)
        p = params or {**self.params}
        last_action = None
        
        while curr:
            curr.set_params(p)
            # DIFFERENT: checks if node is AsyncNode
            if isinstance(curr, AsyncNode):
                last_action = await curr._run_async(shared)
            else:
                last_action = curr._run(shared)
            curr = copy.copy(self.get_next_node(curr, last_action))
        
        return last_action
    
    async def _run_async(self, shared):
        p = await self.prep_async(shared)
        o = await self._orch_async(shared)
        return await self.post_async(shared, p, o)
    
    async def post_async(self, shared, prep_res, exec_res):
        return exec_res
```

**Key Insight**: `isinstance(curr, AsyncNode)` determines whether to call `_run_async()` or `_run()`.

---

## AsyncParallelBatchNode Internals

```python
class AsyncParallelBatchNode(AsyncNode, BatchNode):
    async def _exec(self, items):
        # Uses asyncio.gather for parallel execution
        return await asyncio.gather(*(
            super(AsyncParallelBatchNode, self)._exec(i) for i in items
        ))
```

---

## AsyncParallelBatchFlow Internals

```python
class AsyncParallelBatchFlow(AsyncFlow, BatchFlow):
    async def _run_async(self, shared):
        pr = await self.prep_async(shared) or []
        # Uses asyncio.gather for parallel flow execution
        await asyncio.gather(*(
            self._orch_async(shared, {**self.params, **bp}) for bp in pr
        ))
        return await self.post_async(shared, pr, None)
```

---

## Shared Store Semantics

The `shared` parameter is a `dict` that is:
- Passed to all nodes via `prep(shared)` and `_run(shared)`
- Used to pass data between nodes
- Mutable - nodes can modify it directly

```python
# Example flow
def prep(self, shared):
    return shared["input_data"]  # Read from shared

def post(self, shared, prep_res, exec_res):
    shared["output_data"] = exec_res  # Write to shared
    return "next_action"
```

---

## Common Action Strings

| Action | Meaning |
|--------|---------|
| `"default"` | Default next node (used by `>>` operator) |
| `"search"` | Agent decided to search |
| `"answer"` | Agent decided to answer directly |
| `"decide"` | Return to decision node (loop) |
| `"retry"` | Supervisor wants to retry |
| `"continue"` | Multi-agent loop continues |
| `"end"` | Flow/agent terminates |
| `"done"` | Same as end |
| Any custom string | User-defined routing action |

---

## Reference Examples

### Agent Pattern (pocketflow-agent)

```python
def create_agent_flow():
    decide = DecideAction()
    search = SearchWeb()
    answer = AnswerQuestion()
    
    # Wiring
    decide - "search" >> search   # If decide returns "search" → go to search
    decide - "answer" >> answer  # If decide returns "answer" → go to answer
    search - "decide" >> decide  # Loop back to decide
    
    return Flow(start=decide)
```

### Supervisor Pattern (pocketflow-supervisor)

```python
def create_agent_flow():
    agent_flow = create_agent_inner_flow()
    supervisor = SupervisorNode()
    
    agent_flow >> supervisor   # After agent → go to supervisor
    supervisor - "retry" >> agent_flow  # If reject → loop back
    
    return Flow(start=agent_flow)
```

### Map-Reduce Pattern (pocketflow-map-reduce)

```python
def create_resume_processing_flow():
    read_resumes = ReadResumesNode()
    evaluate = EvaluateResumesNode()  # BatchNode
    reduce_results = ReduceResultsNode()
    
    read_resumes >> evaluate >> reduce_results
    
    return Flow(start=read_resumes)
```

### Multi-Agent Pattern (pocketflow-multi-agent)

```python
async def main():
    shared = {
        "hinter_queue": asyncio.Queue(),
        "guesser_queue": asyncio.Queue(),
        # ...
    }
    
    hinter = AsyncHinter()
    guesser = AsyncGuesser()
    
    # Each agent loops back to itself
    hinter_flow = AsyncFlow(start=hinter)
    guesser_flow = AsyncFlow(start=guesser)
    
    hinter - "continue" >> hinter
    guesser - "continue" >> guesser
    
    await asyncio.gather(
        hinter_flow.run_async(shared),
        guesser_flow.run_async(shared)
    )
```

---

## Key Patterns Summary

| Pattern | How It's Built |
|---------|---------------|
| Sequential | `node_a >> node_b` |
| Conditional | `node - "action" >> target` |
| Loop | `node_b - "action" >> node_a` (later wires back to earlier) |
| Batch | `BatchNode` with `prep()` returning iterable |
| Async | `AsyncNode` with `_run_async()` |
| Parallel | `asyncio.gather()` over multiple flows |
| Nested | `Flow` as successor of another node |
| Heartbeat | Outer Flow looping with inner Flow per cycle |
| Self-Healing | Error feedback loop with max attempts |
| Guardrails | Validation with retry loop |
| Majority Vote | `BatchNode` with counting aggregation |
| Judge/Supervisor | Validation node with retry back to agent |

---

## Additional Patterns Discovered

### Heartbeat Monitor Pattern

```python
# From pocketflow-heartbeat/
class WaitNode(Node):
    def exec(self, prep_res):
        time.sleep(self.poll_interval)
        shared['cycle_count'] += 1
        if shared['cycle_count'] >= shared.get('max_cycles', 5):
            return "done"
        return "continue"

# Outer loop
wait_node >> email_flow
email_flow >> wait_node
wait_node - "done" >> end_node
```

### Self-Healing Pattern

```python
# From pocketflow-self-healing-mermaid/
class CompileChart(Node):
    def exec(self, code):
        result = subprocess.run(["npx", "mermaid-cli", ...])
        if result.returncode != 0:
            return {"success": False, "error": result.stderr}
        return {"success": True}
    
    def post(self, shared, exec_res):
        if not exec_res["success"]:
            shared["attempts"].append({"code": shared["chart"], "error": exec_res["error"]})
            if len(shared["attempts"]) >= 3:
                return "done"
            return "fix"  # Loop back with error
        return "done"
```

### Guardrails Pattern

```python
# From pocketflow-chat-guardrail/
class GuardrailNode(Node):
    def exec(self, user_input):
        is_valid, reason = llm_validate(user_input)
        return is_valid, reason
    
    def post(self, shared, exec_res):
        is_valid, message = exec_res
        if not is_valid:
            return "retry"
        return "process"
```

### Majority Vote Pattern

```python
# From pocketflow-majority-vote/
class MajorityVoteNode(BatchNode):
    def prep(self, shared):
        return [shared["question"] for _ in range(shared["num_tries"])]
    
    def exec_fallback(self, prep_res, exc):
        return None  # Graceful fallback
    
    def post(self, shared, prep_res, exec_res_list):
        counter = collections.Counter([r for r in exec_res_list if r is not None])
        shared["majority_answer"] = counter.most_common(1)[0][0]
        return "end"
```
