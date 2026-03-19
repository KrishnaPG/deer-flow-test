import { proxy } from 'valtio';
import type { Template, InstanceDetails } from '../definitions';

export type Mode = 'deploy' | 'existing';
export type TestStatus = 'idle' | 'testing' | 'success' | 'error';
export type ConsulStatus = 'connected' | 'disconnected' | 'checking';
export type ConsoleMode = 'deployment' | 'container';

export interface TabState {
  id: string;
  name: string;
  mode: Mode;
  formValues: Record<string, string>;
  selectedHost: string;
  existingUrl: string;
  testStatus: TestStatus;
  testMessage?: string;
  instanceDetails: InstanceDetails | null;
  isProcessing: boolean;
}

export const store = proxy({
  // === Server State (synced from TanStack Query) ===
  templates: [] as Template[],
  configs: {} as Record<string, string>,
  hosts: [] as string[],
  
  isLoadingTemplates: false,
  isLoadingConfigs: false,
  templatesError: null as string | null,
  configsError: null as string | null,
  
  // === Client State (pure Valtio) ===
  tabs: [] as TabState[],
  activeTabId: null as string | null,
  
  // Settings (persisted)
  consulUrl: '',
  deployPath: '',
  
  // Operation states
  deploying: null as string | null,
  showSettings: false,
  settingsInputs: { consulUrl: '', deployPath: '' },
  
  // Logs: serviceId -> log lines
  logs: {} as Record<string, string[]>,
  consoleMode: {} as Record<string, ConsoleMode>,
  
  // WebSocket
  wsConnected: false,
  
  // CONSUL status
  consulStatus: 'checking' as ConsulStatus,
  retryingConnection: false,
  consulError: null as string | null,
});

export type Store = typeof store;

// Re-export all selectors
export {
  selectSidebarState,
  selectTabContentState,
  selectServiceStatus,
  selectActiveTab,
  selectActiveTemplate,
  selectTabLogs,
  selectTabConsoleMode,
  selectIsServiceManaged,
  selectCanRegisterExisting,
  type SidebarStateKey,
  type TabContentStateKey,
} from './selectors';
