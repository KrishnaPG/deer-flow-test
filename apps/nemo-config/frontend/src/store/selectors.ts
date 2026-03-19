// === Component State Selectors (NO branching in components) ===
// These selectors work with the frozen snapshot from useSnapshot

import { CONSUL_PREFIX } from '../../../schema';

export type SidebarStateKey = 'loading' | 'error' | 'empty' | 'list';

export const selectSidebarState = (s: any): SidebarStateKey => {
  if (s.isLoadingTemplates) return 'loading';
  if (s.templatesError) return 'error';
  if (s.templates.length === 0) return 'empty';
  return 'list';
};

export type TabContentStateKey = 
  | 'error'
  | 'checking' 
  | 'healthy'
  | 'loading-details'
  | 'container-not-found'
  | 'deploy'
  | 'existing';

const getServiceUrlKey = (serviceId: string) => `${CONSUL_PREFIX}.${serviceId}.url`;

export const selectTabContentState = (s: any, tabId: string | null): TabContentStateKey => {
  if (!tabId) return 'deploy';

  if (s.configsError) return 'error';
  if (s.isLoadingConfigs && !s.configs[getServiceUrlKey(tabId)]) return 'checking';
  if (s.configs[getServiceUrlKey(tabId)]) {
    const tab = s.tabs.find((t: any) => t.id === tabId);
    const instanceDetails = tab?.instanceDetails;
    const containerStatus = instanceDetails?.containerStatus;
    const managedBy = instanceDetails?.metadata?.managedBy;
    
    if (managedBy === 'external') {
      return 'healthy';
    }
    if (managedBy === 'nemo' && containerStatus === 'not_found') {
      return 'container-not-found';
    }
    if (s.isLoadingInstanceDetails || !instanceDetails) {
      return 'loading-details';
    }
    return 'healthy';
  }
  const tab = s.tabs.find((t: any) => t.id === tabId);
  return tab?.mode || 'deploy';
};

export const selectServiceStatus = (s: any, serviceId: string): string => {
  if (s.configs[getServiceUrlKey(serviceId)]) return 'healthy';
  if (s.deploying === serviceId) return 'deploying';
  return 'unconfigured';
};

export const selectActiveTab = (s: any): any | null => {
  return s.tabs.find((t: any) => t.id === s.activeTabId) || null;
};

export const selectActiveTemplate = (s: any): any | null => {
  return s.templates.find((t: any) => t.id === s.activeTabId) || null;
};

export const selectTabLogs = (s: any, tabId: string | null): string[] => {
  if (!tabId) return [];
  return s.logs[tabId] || [];
};

export const selectTabConsoleMode = (s: any, tabId: string | null): string => {
  if (!tabId) return 'deployment';
  return s.consoleMode[tabId] || 'deployment';
};

export const selectIsServiceManaged = (s: any, tabId: string | null): boolean => {
  if (!tabId) return false;
  const tab = s.tabs.find((t: any) => t.id === tabId);
  return tab?.instanceDetails?.metadata?.managedBy === 'nemo';
};

export const selectCanRegisterExisting = (s: any, tabId: string | null): boolean => {
  if (!tabId) return false;
  const tab = s.tabs.find((t: any) => t.id === tabId);
  return tab?.testStatus === 'success';
};
