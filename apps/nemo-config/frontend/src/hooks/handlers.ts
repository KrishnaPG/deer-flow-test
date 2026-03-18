import axios from 'axios';
import type { Template } from '../definitions';
import type { TabState } from './useTabs';
import { 
  fetchInstanceDetails, 
  fetchContainerLogs
} from '../api/services';

const API_URL = 'http://localhost:3001/api';

export interface HandlersConfig {
  tabs: TabState[];
  templates: Template[];
  natsUrl: string;
  deployPath: string;
  updateTab: (tabId: string, updates: Partial<TabState>) => void;
  appendConsole: (tabId: string, message: string) => void;
  setDeploying: (id: string | null) => void;
  refetchConfigs: () => Promise<unknown>;
}

export function createHandlers(config: HandlersConfig) {
  const { 
    tabs, templates, natsUrl, deployPath,
    updateTab, appendConsole, setDeploying, refetchConfigs
  } = config;

  const handleTestConnection = async (tabId: string) => {
    const tab = tabs.find(t => t.id === tabId);
    const template = templates.find(t => t.id === tabId);
    if (!tab || !template) return;

    if (!tab.existingUrl.trim()) {
      updateTab(tabId, { testStatus: 'error', testMessage: 'Please enter a connection URL' });
      appendConsole(tabId, 'Error: No connection URL provided');
      return;
    }

    updateTab(tabId, { testStatus: 'testing', testMessage: undefined });
    appendConsole(tabId, `Testing connection to ${tab.existingUrl}...`);

    const healthCheck = template.health_check;
    if (!healthCheck) {
      updateTab(tabId, { testStatus: 'error', testMessage: 'No health check configuration' });
      appendConsole(tabId, 'Error: No health check configuration found');
      return;
    }

    try {
      const res = await axios.post(`${API_URL}/test-connection`, {
        service_id: tabId,
        connection_url: tab.existingUrl,
        health_check: healthCheck,
        metadata: tab.formValues
      }, { timeout: 15000 });

      if (res.data.success) {
        updateTab(tabId, { 
          testStatus: 'success', 
          testMessage: res.data.message || 'Connection successful' 
        });
        appendConsole(tabId, `✓ ${res.data.message}`);
      } else {
        updateTab(tabId, { 
          testStatus: 'error', 
          testMessage: res.data.message || 'Connection failed' 
        });
        appendConsole(tabId, `✗ ${res.data.message}`);
      }
    } catch (err) {
      const errorMsg = axios.isAxiosError(err) 
        ? err.response?.data?.error || err.message 
        : 'Connection test failed';
      updateTab(tabId, { testStatus: 'error', testMessage: errorMsg });
      appendConsole(tabId, `✗ Error: ${errorMsg}`);
    }
  };

  const handleDeploy = async (tabId: string) => {
    const tab = tabs.find(t => t.id === tabId);
    const template = templates.find(t => t.id === tabId);
    if (!tab || !template) return;

    if (tab.mode === 'existing' && tab.testStatus !== 'success') {
      appendConsole(tabId, 'Error: Please test the connection successfully before registering');
      return;
    }

    setDeploying(tabId);
    appendConsole(tabId, `Starting ${tab.mode} operation...`);

    try {
      if (tab.mode === 'existing') {
        await axios.post(`${API_URL}/register-existing`, {
          service_id: tabId,
          connection_url: tab.existingUrl,
          nats_url: natsUrl,
          template,
          env_values: tab.formValues
        });
        appendConsole(tabId, 'Successfully registered existing instance');
      } else {
        await axios.post(`${API_URL}/deploy`, {
          target_host: tab.selectedHost,
          service_id: tabId,
          template,
          env_values: tab.formValues,
          nats_url: natsUrl,
          mode: 'deploy',
          deploy_path: deployPath
        });
        appendConsole(tabId, 'Deployment completed successfully');
      }

      await refetchConfigs();
      
      try {
        const details = await fetchInstanceDetails(tabId, natsUrl);
        
        if (details.metadata?.managedBy === 'nemo') {
          const logs = await fetchContainerLogs(tabId, natsUrl);
          updateTab(tabId, { 
            instanceDetails: details,
            consoleOutput: logs,
            consoleMode: 'container'
          });
        } else {
          updateTab(tabId, { instanceDetails: details });
        }
      } catch (err) {
        console.error('Failed to fetch instance details after deploy', err);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Deployment failed';
      appendConsole(tabId, `Error: ${errorMessage}`);
    } finally {
      setDeploying(null);
    }
  };

  const handleContainerAction = async (tabId: string, action: 'stop' | 'start' | 'restart' | 'delete' | 'removeConfig') => {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return;

    updateTab(tabId, { isProcessing: true });
    appendConsole(tabId, `${action} operation in progress...`);

    try {
      const endpoint = action === 'removeConfig' 
        ? `${API_URL}/services/${tabId}/config`
        : action === 'delete'
          ? `${API_URL}/services/${tabId}/container`
          : `${API_URL}/services/${tabId}/${action}`;
      
      const method = action === 'delete' || action === 'removeConfig' ? 'delete' : 'post';
      
      await axios({ method, url: endpoint, params: { nats_url: natsUrl, deploy_path: deployPath } });

      appendConsole(tabId, `${action} completed successfully`);

      if (action === 'delete' || action === 'removeConfig') {
        updateTab(tabId, { 
          instanceDetails: null, 
          consoleOutput: [],
          consoleMode: 'deployment'
        });
        await refetchConfigs();
      } else {
        setTimeout(async () => {
          try {
            const details = await fetchInstanceDetails(tabId, natsUrl);
            updateTab(tabId, { instanceDetails: details });
          } catch (err) {
            console.error('Failed to refresh instance details', err);
          }
        }, 1000);
      }
    } catch (err) {
      const errorMsg = axios.isAxiosError(err) 
        ? err.response?.data?.error || err.message 
        : 'Operation failed';
      appendConsole(tabId, `Error: ${errorMsg}`);
    } finally {
      updateTab(tabId, { isProcessing: false });
    }
  };

  const handleExport = async () => {
    try {
      const res = await axios.get(`${API_URL}/export-env`, { params: { nats_url: natsUrl } });
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

  return {
    handleTestConnection,
    handleDeploy,
    handleContainerAction,
    handleExport
  };
}