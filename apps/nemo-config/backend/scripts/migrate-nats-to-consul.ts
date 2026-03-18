#!/usr/bin/env bun
/**
 * Migration Script: NATS KV → Consul KV
 * 
 * This script migrates all configuration data from NATS JetStream KV
 * to HashiCorp Consul KV.
 * 
 * Usage:
 *   bun run scripts/migrate-nats-to-consul.ts
 *   bun run scripts/migrate-nats-to-consul.ts --nats-url nats://10.7.0.4:4222 --consul-url 10.7.0.4:8500
 */

import { connect, StringCodec } from "nats";
import Consul from "consul";

const sc = StringCodec();
const DEFAULT_NATS_URL = 'nats://10.7.0.4:4222';
const DEFAULT_CONSUL_URL = '10.7.0.4:8500';

// Type assertion helper for MigrationOptions
function ensureString(value: string | undefined, defaultValue: string): string {
  return value || defaultValue;
}
const KV_BUCKET = "nemo_config";
const CONSUL_PREFIX = "nemo";

interface MigrationOptions {
  natsUrl: string;
  consulUrl: string;
  dryRun: boolean;
  verbose: boolean;
}

function parseArgs(): MigrationOptions {
  const args = process.argv.slice(2);
  const options: MigrationOptions = {
    natsUrl: DEFAULT_NATS_URL,
    consulUrl: DEFAULT_CONSUL_URL,
    dryRun: false,
    verbose: false
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    switch (arg) {
      case '--nats-url':
        options.natsUrl = ensureString(args[++i], DEFAULT_NATS_URL);
        break;
      case '--consul-url':
        options.consulUrl = ensureString(args[++i], DEFAULT_CONSUL_URL);
        break;
      case '--dry-run':
        options.dryRun = true;
        break;
      case '--verbose':
      case '-v':
        options.verbose = true;
        break;
      case '--help':
      case '-h':
        console.log(`
Migration Script: NATS KV → Consul KV

Usage:
  bun run scripts/migrate-nats-to-consul.ts [options]

Options:
  --nats-url <url>     NATS URL (default: ${DEFAULT_NATS_URL})
  --consul-url <url>   Consul URL (default: ${DEFAULT_CONSUL_URL})
  --dry-run            Show what would be migrated without writing
  --verbose, -v        Show detailed output
  --help, -h           Show this help message

Examples:
  bun run scripts/migrate-nats-to-consul.ts
  bun run scripts/migrate-nats-to-consul.ts --nats-url nats://localhost:4222 --consul-url localhost:8500
  bun run scripts/migrate-nats-to-consul.ts --dry-run --verbose
        `);
        process.exit(0);
        break;
    }
  }

  return options;
}

function getConsulClient(consulUrl: string): Consul {
  let url = consulUrl;
  
  if (url.startsWith('http://')) {
    url = url.substring(7);
  } else if (url.startsWith('https://')) {
    url = url.substring(8);
  }
  
  const [host, portStr] = url.includes(':') 
    ? url.split(':') 
    : [url, '8500'];
  
  return new Consul({
    host: host || '10.7.0.4',
    port: parseInt(portStr || '8500', 10),
  }) as any;
}

async function migrate(options: MigrationOptions) {
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║     NATS KV → Consul KV Migration Tool                    ║');
  console.log('╚════════════════════════════════════════════════════════════╝');
  console.log();
  console.log(`Source (NATS):      ${options.natsUrl}`);
  console.log(`Destination (Consul): ${options.consulUrl}`);
  console.log(`Mode:               ${options.dryRun ? 'DRY RUN' : 'LIVE'}`);
  console.log();

  // Connect to NATS
  console.log('[1/4] Connecting to NATS...');
  let nc;
  try {
    nc = await connect({ servers: options.natsUrl });
    console.log('      ✓ Connected to NATS');
  } catch (err: any) {
    console.error('      ✗ Failed to connect to NATS:', err.message);
    process.exit(1);
  }

  // Connect to Consul
  console.log('[2/4] Connecting to Consul...');
  let consul;
  try {
    consul = getConsulClient(options.consulUrl);
    await consul.status.leader();
    console.log('      ✓ Connected to Consul');
  } catch (err: any) {
    console.error('      ✗ Failed to connect to Consul:', err.message);
    await nc.drain();
    process.exit(1);
  }

  // Read all keys from NATS
  console.log('[3/4] Reading data from NATS...');
  const migrationData: Array<{ natsKey: string; consulKey: string; value: string }> = [];
  
  try {
    const js = nc.jetstream();
    const kv = await js.views.kv(KV_BUCKET);
    
    const keysIter = await kv.keys(">");
    let keyCount = 0;
    
    for await (const key of keysIter) {
      const entry = await kv.get(key);
      if (entry && entry.operation !== "DEL" && entry.operation !== "PURGE") {
        const value = sc.decode(entry.value);
        
        // Convert NATS key to Consul key
        // postgres.url -> nemo/postgres/url
        // nemo_metadata.postgres -> nemo/metadata/postgres
        let consulKey = key.replace(/\./g, '/');
        if (!consulKey.startsWith(CONSUL_PREFIX)) {
          consulKey = `${CONSUL_PREFIX}/${consulKey}`;
        }
        // Special case for metadata
        consulKey = consulKey.replace('/nemo_metadata/', '/metadata/');
        
        migrationData.push({ natsKey: key, consulKey, value });
        keyCount++;
        
        if (options.verbose) {
          console.log(`      Found: ${key} → ${consulKey}`);
        }
      }
    }
    
    console.log(`      ✓ Found ${keyCount} keys to migrate`);
  } catch (err: any) {
    console.error('      ✗ Failed to read from NATS:', err.message);
    await nc.drain();
    process.exit(1);
  }

  // Migrate data to Consul
  console.log('[4/4] Migrating data to Consul...');
  let successCount = 0;
  let errorCount = 0;
  
  for (const item of migrationData) {
    try {
      if (!options.dryRun) {
        await consul.kv.set(item.consulKey, item.value);
      }
      successCount++;
      
      if (options.verbose || options.dryRun) {
        const action = options.dryRun ? 'Would write' : 'Wrote';
        console.log(`      ${action}: ${item.natsKey} → ${item.consulKey}`);
        if (options.verbose && item.value.length < 100) {
          console.log(`             Value: ${item.value}`);
        }
      }
    } catch (err: any) {
      errorCount++;
      console.error(`      ✗ Failed to write ${item.consulKey}:`, err.message);
    }
  }

  // Cleanup
  await nc.drain();

  // Summary
  console.log();
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║     Migration Summary                                      ║');
  console.log('╚════════════════════════════════════════════════════════════╝');
  console.log();
  console.log(`Total keys:     ${migrationData.length}`);
  console.log(`Successful:     ${successCount}`);
  console.log(`Failed:         ${errorCount}`);
  console.log();
  
  if (options.dryRun) {
    console.log('⚠️  This was a DRY RUN. No data was actually written.');
    console.log('   Run without --dry-run to perform the actual migration.');
  } else if (errorCount === 0) {
    console.log('✅ Migration completed successfully!');
    console.log();
    console.log('Next steps:');
    console.log('  1. Verify data in Consul: curl http://10.7.0.4:8500/v1/kv/nemo/?recurse');
    console.log('  2. Update your applications to use Consul instead of NATS');
    console.log('  3. Test the nemo-config backend with the new Consul backend');
  } else {
    console.log('⚠️  Migration completed with errors.');
    console.log('   Please check the error messages above.');
  }
  console.log();
}

// Run migration
const options = parseArgs();
migrate(options).catch(err => {
  console.error('Migration failed:', err);
  process.exit(1);
});
