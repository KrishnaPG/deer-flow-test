import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { cleanupTestResources } from './helpers/consul';
import { get } from './helpers/api';
import { CONFIG } from './config';

describe('Catalog API - E2E Tests', () => {
  beforeAll(async () => {
    // Clean up any leftover test resources
    await cleanupTestResources();
  });

  afterAll(async () => {
    // Final cleanup
    await cleanupTestResources();
  });

  it('should return all service templates with required fields', async () => {
    const { data, status } = await get('/api/catalog');
    expect(status).toBe(200);
    expect(Array.isArray(data)).toBe(true);
    
    // Should have all 8 templates
    expect(data.length).toBe(8);
    
    const templateIds = data.map((t: any) => t.id);
    expect(templateIds).toContain('redis');
    expect(templateIds).toContain('postgres');
    expect(templateIds).toContain('nats');
    expect(templateIds).toContain('minio');
    expect(templateIds).toContain('clickhouse');
    expect(templateIds).toContain('temporal');
    expect(templateIds).toContain('livekit');
    expect(templateIds).toContain('signoz');

    // Validate each template has required fields
    for (const template of data) {
      expect(template).toHaveProperty('id');
      expect(template).toHaveProperty('name');
      expect(template).toHaveProperty('icon');
      expect(template).toHaveProperty('default_port');
      expect(template).toHaveProperty('connection_url_pattern');
      expect(template).toHaveProperty('env_vars');
      expect(template).toHaveProperty('health_check');
      expect(template).toHaveProperty('docker_compose');
      expect(template).toHaveProperty('exports');
      
      // Validate types
      expect(typeof template.id).toBe('string');
      expect(typeof template.name).toBe('string');
      expect(typeof template.icon).toBe('string');
      expect(typeof template.default_port).toBe('number');
      expect(Array.isArray(template.env_vars)).toBe(true);
      expect(typeof template.health_check).toBe('object');
      expect(typeof template.docker_compose).toBe('string');
      expect(typeof template.exports).toBe('object');
    }
  });

  it('should have correct exports for postgres template', async () => {
    const { data } = await get('/api/catalog');
    const postgresTemplate = data.find((t: any) => t.id === 'postgres');
    
    expect(postgresTemplate).toBeDefined();
    expect(postgresTemplate.exports['postgres.url']).toBeTruthy();
    expect(postgresTemplate.exports['postgres.user']).toBeTruthy();
    expect(postgresTemplate.exports['postgres.password']).toBeTruthy();
    expect(postgresTemplate.exports['postgres.database']).toBeTruthy();
    
    // Check that the export patterns use correct placeholders
    expect(postgresTemplate.exports['postgres.url']).toContain('${HOST}');
    expect(postgresTemplate.exports['postgres.url']).toContain('5432');
    expect(postgresTemplate.exports['postgres.user']).toContain('POSTGRES_USER');
    expect(postgresTemplate.exports['postgres.password']).toContain('POSTGRES_PASSWORD');
    expect(postgresTemplate.exports['postgres.database']).toContain('POSTGRES_DB');
  });

  it('should have correct health checks for each template', async () => {
    const { data } = await get('/api/catalog');
    
    // Check specific templates have expected health check types
    const templatesById = Object.fromEntries(data.map((t: any) => [t.id, t]));
    
    // Redis should have TCP health check
    expect(templatesById.redis.health_check.type).toBe('tcp');
    expect(templatesById.redis.health_check.port).toBe(6379);
    
    // Postgres should have TCP health check
    expect(templatesById.postgres.health_check.type).toBe('tcp');
    expect(templatesById.postgres.health_check.port).toBe(5432);
    
    // Minio should have HTTP health check
    expect(templatesById.minio.health_check.type).toBe('http');
    expect(templatesById.minio.health_check.port).toBe(9000);
    expect(templatesById.minio.health_check.path).toBe('/minio/health/live');
    
    // Clickhouse should have HTTP health check
    expect(templatesById.clickhouse.health_check.type).toBe('http');
    expect(templatesById.clickhouse.health_check.port).toBe(8123);
    expect(templatesById.clickhouse.health_check.path).toBe('/ping');
  });
});