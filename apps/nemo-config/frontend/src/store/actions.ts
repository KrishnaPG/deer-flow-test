import { store } from './index';
import type { TabState } from './index';
import type { Template, InstanceDetails } from '../definitions';
import { fetchInstanceDetails, fetchContainerLogs } from '../api/services';

// === Tab Management ===

const createDefaultTab = (template: Template): TabState => {
  const formValues: Record<string, string> = {};
  template.env_vars.forEach(env => {
    if (env.default) formValues[env.key] = env.default;
  });

  let existingUrl = '';
  if (template.connection_url_pattern) {
    existingUrl = template.connection_url_pattern
      .replace('${HOST}', 'localhost')
      .replace(/\$\{([^}]+)\}/g, (_, key) => formValues[key] || '');
  }

  return {
    id: template.id,
    name: template.name,
    mode: 'deploy',
    formValues,
    selectedHost: 'localhost',
    existingUrl,
    testStatus: 'idle',
    testMessage: undefined,
    instanceDetails: null,
    isProcessing: false,
  };
};

export const openTab = (template: Template): void => {
  const existingTab = store.tabs.find(t => t.id === template.id);
  if (existingTab) {
    store.activeTabId = template.id;
    return;
  }

  const newTab = createDefaultTab(template);
  store.tabs = [...store.tabs, newTab];
  store.activeTabId = template.id;
};

export const closeTab = (tabId: string): void => {
  const newTabs = store.tabs.filter(t => t.id !== tabId);
  store.tabs = newTabs;
  
  if (store.activeTabId === tabId) {
    store.activeTabId = newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null;
  }
};

export const setActiveTab = (tabId: string | null): void => {
  store.activeTabId = tabId;
};

export const updateTab = (tabId: string, updates: Partial<TabState>): void => {
  const tab = store.tabs.find(t => t.id === tabId);
  if (tab) {
    Object.assign(tab, updates);
  }
};

// === Form State ===

export const setTabMode = (tabId: string, mode: 'deploy' | 'existing'): void => {
  updateTab(tabId, { 
    mode, 
    ...(mode === 'existing' ? {} : { testStatus: 'idle', testMessage: undefined })
  });
};

export const setFormValue = (tabId: string, key: string, value: string): void => {
  const tab = store.tabs.find(t => t.id === tabId);
  if (tab) {
    tab.formValues[key] = value;
  }
};

export const setSelectedHost = (tabId: string, host: string): void => {
  updateTab(tabId, { selectedHost: host });
};

export const setExistingUrl = (tabId: string, url: string): void => {
  updateTab(tabId, { existingUrl: url, testStatus: 'idle', testMessage: undefined });
};

// === Test Status ===

export const setTestStatus = (
  tabId: string, 
  status: TabState['testStatus'], 
  message?: string
): void => {
  updateTab(tabId, { testStatus: status, testMessage: message });
};

// === Logs ===

export const appendLog = (serviceId: string, message: string): void => {
  const timestamp = new Date().toLocaleTimeString();
  const logLine = `[${timestamp}] ${message}`;
  const currentLogs = store.logs[serviceId] || [];
  store.logs = { ...store.logs, [serviceId]: [...currentLogs, logLine] };
};

export const clearLogs = (serviceId: string): void => {
  store.logs = { ...store.logs, [serviceId]: [] };
};

export const setConsoleMode = (serviceId: string, mode: 'deployment' | 'container'): void => {
  store.consoleMode = { ...store.consoleMode, [serviceId]: mode };
};

// === Settings ===

export const openSettings = (): void => {
  store.settingsInputs = { consulUrl: store.consulUrl, deployPath: store.deployPath };
  store.showSettings = true;
};

export const closeSettings = (): void => {
  store.showSettings = false;
};

export const updateSettingsInput = (field: 'consulUrl' | 'deployPath', value: string): void => {
  store.settingsInputs = { ...store.settingsInputs, [field]: value };
};

export const saveSettings = (): void => {
  store.consulUrl = store.settingsInputs.consulUrl;
  store.deployPath = store.settingsInputs.deployPath;
  store.showSettings = false;
};

// === Deployment State ===

export const setDeploying = (serviceId: string | null): void => {
  store.deploying = serviceId;
};

export const setProcessing = (tabId: string, isProcessing: boolean): void => {
  updateTab(tabId, { isProcessing });
};

// === Instance Details ===

export const setInstanceDetails = (tabId: string, details: InstanceDetails | null): void => {
  updateTab(tabId, { instanceDetails: details });
};

export const loadInstanceDetails = async (tabId: string, consulUrl: string): Promise<void> => {
  try {
    const details = await fetchInstanceDetails(tabId, consulUrl);
    setInstanceDetails(tabId, details);
  } catch (err) {
    console.error('Failed to fetch instance details:', err);
  }
};

export const loadContainerLogs = async (tabId: string, consulUrl: string): Promise<void> => {
  try {
    const logs = await fetchContainerLogs(tabId, consulUrl);
    store.logs = { ...store.logs, [tabId]: logs };
    store.consoleMode = { ...store.consoleMode, [tabId]: 'container' };
  } catch (err) {
    console.error('Failed to fetch container logs:', err);
  }
};

// === WebSocket ===

export const setWsConnected = (connected: boolean): void => {
  store.wsConnected = connected;
};
