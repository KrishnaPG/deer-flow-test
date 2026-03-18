import { store } from './index';
import * as actions from './actions';
import { queryClient } from '../hooks/useServerSync';
import { 
  fetchInstanceDetails,
  fetchContainerLogs
} from '../api/services';
import axios from 'axios';

const API_URL = 'http://localhost:3001/api';

// === Data Fetching Actions (triggered by TanStack Query sync) ===

export const setTemplates = (templates: Template[]): void => {
  store.templates = templates;
};

export const setConfigs = (configs: Record<string, string>): void => {
  store.configs = configs;
};

export const setHosts = (hosts: string[]): void => {
  store.hosts = hosts;
};

export const setLoadingTemplates = (isLoading: boolean): void => {
  store.isLoadingTemplates = isLoading;
};

export const setLoadingConfigs = (isLoading: boolean): void => {
  store.isLoadingConfigs = isLoading;
};

export const setTemplatesError = (error: string | null): void => {
  store.templatesError = error;
};

export const setConfigsError = (error: string | null): void => {
  store.configsError = error;
};

export const setNatsStatus = (status: 'connected' | 'disconnected' | 'checking'): void => {
  store.natsStatus = status;
};

// === Async API Actions ===

export const testConnection = async (tabId: string): Promise<void> => {
  const tab = store.tabs.find(t => t.id === tabId);
  const template = store.templates.find(t => t.id === tabId);
  
  if (!tab || !template) return;
  
  if (!tab.existingUrl.trim()) {
    actions.setTestStatus(tabId, 'error', 'Please enter a connection URL');
    actions.appendLog(tabId, 'Error: No connection URL provided');
    return;
  }
  
  actions.setTestStatus(tabId, 'testing');
  actions.appendLog(tabId, `Testing connection to ${tab.existingUrl}...`);
  
  try {
    const res = await axios.post(`${API_URL}/test-connection`, {
      service_id: tabId,
      connection_url: tab.existingUrl,
      health_check: template.health_check,
      metadata: tab.formValues,
    }, { timeout: 15000 });
    
    if (res.data.success) {
      actions.setTestStatus(tabId, 'success', res.data.message || 'Connection successful');
      actions.appendLog(tabId, `✓ ${res.data.message}`);
    } else {
      actions.setTestStatus(tabId, 'error', res.data.message || 'Connection failed');
      actions.appendLog(tabId, `✗ ${res.data.message}`);
    }
  } catch (err) {
    const errorMsg = axios.isAxiosError(err) 
      ? err.response?.data?.error || err.message 
      : 'Connection test failed';
    actions.setTestStatus(tabId, 'error', errorMsg);
    actions.appendLog(tabId, `✗ Error: ${errorMsg}`);
  }
};

export const deploy = async (tabId: string): Promise<void> => {
  const tab = store.tabs.find(t => t.id === tabId);
  const template = store.templates.find(t => t.id === tabId);
  
  if (!tab || !template) return;
  
  if (tab.mode === 'existing' && tab.testStatus !== 'success') {
    actions.appendLog(tabId, 'Error: Please test the connection successfully before registering');
    return;
  }
  
  actions.setDeploying(tabId);
  actions.appendLog(tabId, `Starting ${tab.mode} operation...`);
  
  try {
    if (tab.mode === 'existing') {
      await axios.post(`${API_URL}/register-existing`, {
        service_id: tabId,
        connection_url: tab.existingUrl,
        nats_url: store.natsUrl,
        template,
        env_values: tab.formValues,
      });
      actions.appendLog(tabId, '[Register] Successfully registered');
    } else {
      await axios.post(`${API_URL}/deploy`, {
        target_host: tab.selectedHost,
        service_id: tabId,
        template,
        env_values: tab.formValues,
        nats_url: store.natsUrl,
        mode: 'deploy',
        deploy_path: store.deployPath,
      });
      actions.appendLog(tabId, '[Deploy] Deployment completed successfully');
    }
    
    // Invalidate configs query to trigger refetch
    queryClient.invalidateQueries({ queryKey: ['configs'] });
    
    // Load instance details after successful deployment
    try {
      const details = await fetchInstanceDetails(tabId, store.natsUrl);
      actions.setInstanceDetails(tabId, details);
      
      if (details.metadata?.managedBy === 'nemo') {
        const logs = await fetchContainerLogs(tabId, store.natsUrl);
        store.logs = { ...store.logs, [tabId]: logs };
        store.consoleMode = { ...store.consoleMode, [tabId]: 'container' };
      }
    } catch (err) {
      console.error('Failed to fetch instance details after deploy:', err);
    }
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : 'Deployment failed';
    actions.appendLog(tabId, `Error: ${errorMessage}`);
  } finally {
    actions.setDeploying(null);
  }
};

export const handleContainerAction = async (
  tabId: string, 
  action: 'stop' | 'start' | 'restart' | 'delete' | 'removeConfig'
): Promise<void> => {
  const tab = store.tabs.find(t => t.id === tabId);
  if (!tab) return;
  
  actions.setProcessing(tabId, true);
  actions.appendLog(tabId, `${action} operation in progress...`);
  
  try {
    const endpoint = action === 'removeConfig' 
      ? `${API_URL}/services/${tabId}/config`
      : action === 'delete'
        ? `${API_URL}/services/${tabId}/container`
        : `${API_URL}/services/${tabId}/${action}`;
    
    const method = action === 'delete' || action === 'removeConfig' ? 'delete' : 'post';
    
    await axios({ method, url: endpoint, params: { nats_url: store.natsUrl, deploy_path: store.deployPath } });
    
    actions.appendLog(tabId, `${action} completed successfully`);
    
    if (action === 'delete' || action === 'removeConfig') {
      actions.setInstanceDetails(tabId, null);
      actions.clearLogs(tabId);
      store.consoleMode = { ...store.consoleMode, [tabId]: 'deployment' };
      queryClient.invalidateQueries({ queryKey: ['configs'] });
    } else {
      // Refresh instance details after a short delay
      setTimeout(async () => {
        try {
          const details = await fetchInstanceDetails(tabId, store.natsUrl);
          actions.setInstanceDetails(tabId, details);
        } catch (err) {
          console.error('Failed to refresh instance details:', err);
        }
      }, 1000);
    }
  } catch (err) {
    const errorMsg = axios.isAxiosError(err) 
      ? err.response?.data?.error || err.message 
      : 'Operation failed';
    actions.appendLog(tabId, `Error: ${errorMsg}`);
  } finally {
    actions.setProcessing(tabId, false);
  }
};

export const exportConfig = async (): Promise<void> => {
  try {
    const res = await axios.get(`${API_URL}/export-env`, { params: { nats_url: store.natsUrl } });
    const blob = new Blob([res.data], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = '.env.export';
    a.click();
  } catch {
    alert('Failed to export config. Is NATS running?');
  }
};

// Import types
import type { Template } from '../definitions';
