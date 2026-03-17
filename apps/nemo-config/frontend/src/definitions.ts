import { Database, Box, Activity, MessageSquare } from 'lucide-react';

export const ICONS: Record<string, any> = {
  'database': Database,
  'box': Box,
  'activity': Activity,
  'message-square': MessageSquare
};

export interface Template {
  id: string;
  name: string;
  icon: string;
  default_port: number;
  connection_url_pattern?: string;
  env_vars: Array<{
    key: string;
    description: string;
    default?: string;
    secret?: boolean;
  }>;
}

export type Mode = 'deploy' | 'existing';
export type ServiceStatus = 'unconfigured' | 'deploying' | 'healthy';

export type NatsStatus = 'connected' | 'disconnected' | 'checking';

// Types re-exported for Vite HMR
