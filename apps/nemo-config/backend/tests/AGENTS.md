# Nemo-Config Backend E2E Tests

## Overview

End-to-end functional tests for the nemo-config backend API. Tests run against a **real Consul server** and **real Docker deployments** on remote hosts.

## Test Configuration

### Key Infrastructure

| Service | Address | Purpose |
|---------|---------|---------|
| Consul | `10.7.0.4:8500` | Configuration storage |
| Docker | `10.7.0.4` | Container management (via SSH) |
| Backend | `localhost:3001` | API server |

### SSH Access

The backend uses SSH to execute Docker commands on remote hosts. Configure your `~/.ssh/config`:

```ssh-config
Host 10.7.0.4
  User ubuntu
  IdentityFile ~/.ssh/Github_ED25519
```

## Running Tests

```bash
cd apps/nemo-config/backend

# Run all E2E tests
bun test tests/e2e

# Run specific test file
bun test tests/e2e/catalog.test.ts

# Run with concurrency (sequential for docker safety)
bun test tests/e2e --concurrency=1
```

## Test Structure

```
tests/e2e/
├── config.ts              # Test configuration
├── helpers/
│   ├── api.ts            # HTTP client for API
│   ├── consul.ts         # Consul KV operations
│   └── docker.ts        # Docker command executors (local + SSH)
├── catalog.test.ts        # GET /api/catalog
├── consul-health.test.ts  # GET /api/health/consul
├── ssh-hosts.test.ts     # GET /api/ssh-hosts
├── configs.test.ts       # GET /api/configs, GET /api/config/:id
├── export-env.test.ts     # GET /api/export-env
├── services-health.test.ts # GET /api/health/services
├── test-connection.test.ts # POST /api/test-connection
├── register-existing.test.ts # POST /api/register-existing
├── deploy/               # Deploy tests for all 8 templates
│   ├── redis.test.ts
│   ├── postgres.test.ts
│   ├── nats.test.ts
│   ├── minio.test.ts
│   ├── clickhouse.test.ts
│   ├── temporal.test.ts
│   ├── livekit.test.ts
│   └── signoz.test.ts
└── container-ops/        # Container lifecycle tests
    ├── details.test.ts
    ├── logs.test.ts
    ├── stop.test.ts
    ├── start.test.ts
    ├── restart.test.ts
    └── delete.test.ts
```

## Important Notes

### Docker Execution
- Docker commands are executed via SSH on remote hosts (e.g., `ssh 10.7.0.4 "docker ps"`)
- The backend server must have SSH access to remote Docker hosts
- Test containers are named with `nemo-e2e-test-*` prefix

### Consul Connection
- All tests connect to the **real** Consul at `10.7.0.4:8500`
- Tests use the `e2e-test-*` prefix for service IDs to avoid conflicts
- Cleanup runs before/after tests to remove test data

### Test Isolation
- Each test uses a unique timestamped service ID
- Containers are cleaned up after each test
- Consul keys are removed after each test

## API Endpoints Tested

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/catalog` | List all service templates |
| GET | `/api/ssh-hosts` | List available SSH hosts |
| GET | `/api/health/consul` | Check Consul connection |
| GET | `/api/health/services` | Check all services health |
| GET | `/api/configs` | Get all configurations |
| GET | `/api/config/:serviceId` | Get specific service config |
| GET | `/api/export-env` | Export configs as .env |
| POST | `/api/deploy` | Deploy a new container |
| POST | `/api/register-existing` | Register external service |
| POST | `/api/test-connection` | Test service connectivity |
| GET | `/api/services/:id/details` | Get container details |
| GET | `/api/services/:id/logs` | Get container logs |
| POST | `/api/services/:id/stop` | Stop container |
| POST | `/api/services/:id/start` | Start container |
| POST | `/api/services/:id/restart` | Restart container |
| DELETE | `/api/services/:id/container` | Delete container + config |
| DELETE | `/api/services/:id/config` | Remove config only |

## Troubleshooting

### SSH Connection Issues
```bash
# Test SSH connectivity
ssh -o ConnectTimeout=5 10.7.0.4 "echo ok"

# Check SSH agent
ssh-add -l
```

### Consul Connection Issues
```bash
# Test Consul directly
curl http://10.7.0.4:8500/v1/status/leader
```

### Docker Issues
```bash
# Check Docker on remote
ssh 10.7.0.4 "docker ps"

# Check container status
ssh 10.7.0.4 "docker inspect nemo-redis --format='{{.State.Status}}'"
```

## Data Volume Paths (Verified)

All templates support `DATA_PATH` environment variable for persistent data volumes:

| Service | Volume Path | Notes |
|---------|-------------|-------|
| Redis | `/data` | AOF persistence enabled |
| PostgreSQL | `/var/lib/postgresql/data` | PGDATA environment |
| MinIO | `/data` | Object storage data |
| NATS | `/data` | JetStream persistence |
| ClickHouse | `/var/lib/clickhouse` | OLAP data |
| Temporal | `/etc/temporal/config` | Config (uses PostgreSQL for data) |
| LiveKit | `/var/livekit` | Recordings |
| SigNoz | `/var/otel` | OTEL collector data |

### DATA_PATH Behavior
- Default: `./data` (relative to deploy path)
- At runtime: Auto-generated as `{deploy_path}/{serviceId}/data`
- Volume mount: `{DATA_PATH}:{container_path}`
