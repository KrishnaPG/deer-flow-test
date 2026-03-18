export const CONFIG = {
  API_URL: 'http://localhost:3001', // Routes already have /api prefix (see index.ts)
  CONSUL_URL: '10.7.0.4:8500',
  DEPLOY_BASE_PATH: '~/workspace/nemo',
  TEST_PREFIX: 'e2e-test-', // Prefix for test containers and Consul keys to avoid conflicts
  SSH_TIMEOUT: 30000,
  DOCKER_TIMEOUT: 60000,
  // Maximum time to wait for container to be ready (in ms)
  CONTAINER_READY_TIMEOUT: 30000,
};