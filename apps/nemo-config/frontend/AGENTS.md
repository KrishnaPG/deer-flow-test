# Frontend Data Pipeline Architecture

## Core Principles

1. **TanStack Query = Single Source of Truth** for all server data
2. **Valtio Store = UI State** synced from TanStack Query only
3. **useMutations Hook = Single Entry Point** for all mutations
4. **Components = Declarative Rendering** via registry pattern, no if/else

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
│   ├── useWebSocketManager.ts # Real-time deployment logs
│   └── useLogPollingManager.ts # REMOVED (replaced by TanStack Query)
├── store/
│   ├── index.ts              # Valtio state (server + client)
│   ├── selectors.ts          # Component state keys
│   ├── actions.ts            # UI state mutations only
│   ├── api-actions.ts        # REMOVED (replaced by useMutations)
│   └── persistence.ts        # localStorage sync
├── components/
│   └── [Component]/
│       ├── index.tsx         # Registry + state selection
│       └── states/           # Pure UI components per state
└── api/services.ts           # Query/mutation function definitions
```

## Queries (useServerSync)

| Query           | Key                          | Syncs To                | Polling           |
| --------------- | ---------------------------- | ----------------------- | ----------------- |
| templates       | `['templates']`              | `store.templates`       | No                |
| configs         | `['configs', natsUrl]`       | `store.configs`         | No                |
| hosts           | `['hosts']`                  | `store.hosts`           | No                |
| natsHealth      | `['natsHealth', natsUrl]`    | `store.natsStatus`      | 10s               |
| instanceDetails | `['instanceDetails', tabId]` | `store.instanceDetails` | No                |
| containerLogs   | `['containerLogs', tabId]`   | `store.logs[tabId]`     | 1s (managed only) |

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

## Key Rules

1. **NO direct axios calls** in components or actions (except mutationFn)
2. **NO data fetching** outside useServerSync
3. **NO if/else** branching in component render
4. **Mutations ALWAYS** invalidate affected queries
5. **WebSocket ONLY** for real-time deployment logs
6. **TanStack Query ONLY** for all other server data

## Package Manager

Bun