# Frontend Data Pipeline Architecture

## Data Flow

```
User Action
    ↓
Component calls mutation from useMutations()
    ↓
useMutation performs API call
    ↓
onSuccess: invalidateQueries([...])
    ↓
TanStack Query auto-refetches
    ↓
useServerSync detects changes → updates Valtio store
    ↓
Components re-render via useSnapshot()
```

## File Structure

```
frontend/src/
├── hooks/
│   ├── useServerSync.ts      # ALL queries + sync to Valtio
│   ├── useMutations.ts       # ALL mutations + invalidation
│   └── useWebSocketManager.ts # Real-time deployment logs
├── store/
│   ├── index.ts              # Valtio state (server + client)
│   ├── selectors.ts          # Component state keys
│   ├── actions.ts            # UI state mutations only
│   └── persistence.ts        # localStorage sync
├── components/
│   └── [Component]/
│       ├── index.tsx         # Registry + state selection
│       └── states/           # Pure UI components per state
└── api/services.ts           # Query/mutation function definitions
```

## Queries (useServerSync)

| Query           | Key                             | Syncs To                | Polling           |
| --------------- | ------------------------------- | ----------------------- | ----------------- |
| templates       | `['templates']`                 | `store.templates`       | No                |
| configs         | `['configs', consulUrl]`        | `store.configs`         | No                |
| hosts           | `['hosts']`                     | `store.hosts`           | No                |
| consulHealth    | `['consulHealth', consulUrl]`   | `store.consulStatus`    | 10s               |
| servicesHealth  | `['servicesHealth', consulUrl]` | `store.servicesHealth`  | 30s               |
| instanceDetails | `['instanceDetails', tabId]`    | `store.instanceDetails` | No                |
| containerLogs   | `['containerLogs', tabId]`      | `store.logs[tabId]`     | 1s (managed only) |

## Mutations (useMutations)

```typescript
const mutations = {
  deploy: (tabId) => invalidate(['configs', 'instanceDetails', 'containerLogs']),
  registerExisting: (tabId) => invalidate(['configs', 'instanceDetails']),
  testConnection: (tabId) => no invalidation (local only),
  containerAction: (tabId, action) => invalidate(['instanceDetails', 'containerLogs']),
}
```

## Logs Strategy

| Phase             | Source         | Stored In                 | Console Mode |
| ----------------- | -------------- | ------------------------- | ------------ |
| During Deployment | WebSocket      | `store.logs[serviceId]`   | "deployment" |
| After (Managed)   | TanStack Query | `store.logs[serviceId]`   | "container"  |
| After (External)  | N/A            | Keep last or show message | "deployment" |

## Component Rendering

```typescript
// No if/else in components
const Component = () => {
  const snap = useSnapshot(store)
  const stateKey = selectState(snap)  // 'loading' | 'error' | 'healthy'...
  const StateComponent = registry[stateKey]
  return <StateComponent />
}
```

## State Management & Data Flow Strict Rules

The frontend must adhere to strict state separation and mutation rules:

### Server State (TanStack Query)
* **Tool:** `useQuery` and `useMutation` (TanStack Query).
* **Rule:** This is the *single source of truth* for all remote data. Valtio must NEVER cache or duplicate server responses. All data fetching, caching, and invalidation is handled strictly by TanStack.

### Client State (Valtio)
* **Tool:** `valtio` (Proxy-based state).
* **Scope:** 
  1. UI state (e.g. selected nodes, current timestep, active brush windows)
  2. client-side derived calculations based on server data.  (e.g. pivoted results )
* **Access Rules:**
  1. **Reactive:** React components MUST use `useSnapshot(store.part)` to access only the slice of state they need to trigger re-renders.
  2. **Raw/Non-Reactive:** For callbacks or pure functions needing data without triggering renders, read directly from the `valtio` proxy (`store.part`).
  3. **Mutations:** State mutations MUST occur through dedicated, named action methods on the store object itself. No inline or direct property reassignment inside components.

## Coding Constraints

To maintain a pristine, highly-modular codebase, the following constraints are non-negotiable:

1. **Size Limits:**
   * Files must be **< 400 LOC**.
   * React components and functions must be **< 50 LOC**.
2. **No nested or inline Functions:** Callbacks (`onClick`, `onChange`, etc.) must be defined outside the JSX return block as named functions.
3. **Component Registry:** Use Component Registry dictionaries to dynamically resolve and load components.
4. **No `if..else` Rendering:** 
   * Avoid `if...else` branching in JSX rendering logic. 
   * Use the Component Registry, polymorphism, or early returns to handle conditional UI states. 
5. Avoid inlining custom styles for components. Use CSS modules with class reuse. Prioritize Relative Units (rem) for all font sizing to ensure scalability.
   
7. **NO data fetching** outside useServerSync
9.  **Mutations ALWAYS** invalidate affected queries
10. **WebSocket ONLY** for real-time deployment logs
11. **TanStack Query ONLY** for all other server data


## Package Manager

Bun