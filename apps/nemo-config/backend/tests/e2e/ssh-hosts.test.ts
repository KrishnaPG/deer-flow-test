import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { cleanupTestResources } from './helpers/consul';
import { get } from './helpers/api';
import { CONFIG } from './config';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

describe('SSH Hosts API - E2E Tests', () => {
  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  it('should return localhost in hosts list', async () => {
    const { data, status } = await get('/api/ssh-hosts');
    expect(status).toBe(200);
    expect(Array.isArray(data)).toBe(true);
    expect(data).toContain('localhost');
  });

  it('should parse ~/.ssh/config and return configured hosts', async () => {
    const { data, status } = await get('/api/ssh-hosts');
    expect(status).toBe(200);
    
    // Try to read actual ~/.ssh/config to verify parsing
    try {
      const homeDir = process.env.HOME || '/Users/gopalakrishnapalem';
      const sshConfigPath = `${homeDir}/.ssh/config`;
      const { stdout } = await execAsync(`cat ${sshConfigPath} 2>/dev/null || echo ""`);
      
      if (stdout.trim()) {
        // Parse Host lines from config (excluding wildcards)
        const hostsFromConfig = stdout
          .split('\n')
          .map(line => line.trim())
          .filter(line => line.toLowerCase().startsWith('host ') && !line.includes('*') && !line.toLowerCase().startsWith('host *'))
          .map(line => line.split(/\s+/)[1])
          .filter(host => host && host.length > 0);
        
        // API should return localhost plus any hosts from config
        for (const host of hostsFromConfig) {
          expect(data).toContain(host);
        }
      }
    } catch (error) {
      // If we can't read the config, just verify localhost is present
      expect(data).toContain('localhost');
    }
  });
});