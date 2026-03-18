export const CONFIG = {
  API_URL: 'http://localhost:3001',
  CONSUL_URL: '10.7.0.4:8500',
  DEPLOY_BASE_PATH: '~/workspace/nemo',
  TEST_PREFIX: 'e2e-test-',
  DEFAULT_TARGET_HOST: '10.7.0.4', // Remote server with Docker
  SSH_TIMEOUT: 30000,
  DOCKER_TIMEOUT: 60000,
  CONTAINER_READY_TIMEOUT: 60000, // Increased for remote Docker
};