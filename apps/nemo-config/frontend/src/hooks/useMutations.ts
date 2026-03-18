import { useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import { store } from '../store';
import * as actions from '../store/actions';
import type { Template } from '../definitions';

const API_URL = 'http://localhost:3001/api';

interface DeployPayload {
  target_host: string;
  service_id: string;
  template: Template;
  env_values: Record<string, string>;
  nats_url: string;
  mode: 'deploy';
  deploy_path: string;
}

interface RegisterExistingPayload {
  service_id: string;
  connection_url: string;
  nats_url: string;
  template: Template;
  env_values: Record<string, string>;
}

interface TestConnectionPayload {
  service_id: string;
  connection_url: string;
  health_check: Template['health_check'];
  metadata: Record<string, string>;
}

const testConnectionFn = async (payload: TestConnectionPayload) => {
  const res = await axios.post(`${API_URL}/test-connection`, payload, {
    timeout: 15000,
  });
  return res.data;
};

const deployFn = async (payload: DeployPayload) => {
  const res = await axios.post(`${API_URL}/deploy`, payload);
  return res.data;
};

const registerExistingFn = async (payload: RegisterExistingPayload) => {
  const res = await axios.post(`${API_URL}/register-existing`, payload);
  return res.data;
};

const containerActionFn = async ({
  tabId,
  action,
  natsUrl,
  deployPath,
}: {
  tabId: string;
  action: 'stop' | 'start' | 'restart' | 'delete' | 'removeConfig';
  natsUrl: string;
  deployPath: string;
}) => {
  const endpoint =
    action === 'removeConfig'
      ? `${API_URL}/services/${tabId}/config`
      : action === 'delete'
        ? `${API_URL}/services/${tabId}/container`
        : `${API_URL}/services/${tabId}/${action}`;

  const method = action === 'delete' || action === 'removeConfig' ? 'delete' : 'post';

  const res = await axios({
    method,
    url: endpoint,
    params: { nats_url: natsUrl, deploy_path: deployPath },
  });
  return res.data;
};

const exportConfigFn = async (natsUrl: string) => {
  const res = await axios.get(`${API_URL}/export-env`, {
    params: { nats_url: natsUrl },
  });
  return res.data;
};

export const useMutations = () => {
  const queryClient = useQueryClient();

  const testConnection = useMutation({
    mutationFn: testConnectionFn,
    onMutate: (variables) => {
      actions.setTestStatus(variables.service_id, 'testing');
      actions.appendLog(variables.service_id, `Testing connection to ${variables.connection_url}...`);
    },
    onSuccess: (data, variables) => {
      if (data.success) {
        actions.setTestStatus(variables.service_id, 'success', data.message || 'Connection successful');
        actions.appendLog(variables.service_id, `✓ ${data.message}`);
      } else {
        actions.setTestStatus(variables.service_id, 'error', data.message || 'Connection failed');
        actions.appendLog(variables.service_id, `✗ ${data.message}`);
      }
    },
    onError: (error: unknown, variables) => {
      const errorMsg = axios.isAxiosError(error)
        ? error.response?.data?.error || error.message
        : 'Connection test failed';
      actions.setTestStatus(variables.service_id, 'error', errorMsg);
      actions.appendLog(variables.service_id, `✗ Error: ${errorMsg}`);
    },
  });

  const executeDeploy = useMutation({
    mutationFn: async (tabId: string) => {
      const tab = store.tabs.find((t) => t.id === tabId);
      const template = store.templates.find((t) => t.id === tabId);

      if (!tab || !template) {
        throw new Error('Tab or template not found');
      }

      if (tab.mode === 'existing' && tab.testStatus !== 'success') {
        throw new Error('Please test the connection successfully before registering');
      }

      if (tab.mode === 'existing') {
        return registerExistingFn({
          service_id: tabId,
          connection_url: tab.existingUrl,
          nats_url: store.natsUrl,
          template,
          env_values: tab.formValues,
        });
      } else {
        return deployFn({
          target_host: tab.selectedHost,
          service_id: tabId,
          template,
          env_values: tab.formValues,
          nats_url: store.natsUrl,
          mode: 'deploy',
          deploy_path: store.deployPath,
        });
      }
    },
    onMutate: (tabId) => {
      const tab = store.tabs.find((t) => t.id === tabId);
      const mode = tab?.mode || 'deploy';
      actions.setDeploying(tabId);
      actions.appendLog(tabId, `Starting ${mode} operation...`);
    },
    onSuccess: (_, tabId) => {
      const tab = store.tabs.find((t) => t.id === tabId);
      const mode = tab?.mode || 'deploy';
      if (mode === 'existing') {
        actions.appendLog(tabId, '[Register] Successfully registered');
      } else {
        actions.appendLog(tabId, '[Deploy] Deployment completed successfully');
      }
      actions.setDeploying(null);
      queryClient.invalidateQueries({ queryKey: ['configs'] });
      queryClient.invalidateQueries({ queryKey: ['instanceDetails', tabId] });
      queryClient.invalidateQueries({ queryKey: ['containerLogs', tabId] });
    },
    onError: (error: unknown, tabId) => {
      const errorMessage = error instanceof Error ? error.message : 'Operation failed';
      actions.appendLog(tabId, `Error: ${errorMessage}`);
      actions.setDeploying(null);
    },
  });

  const containerAction = useMutation({
    mutationFn: containerActionFn,
    onMutate: (variables) => {
      actions.setProcessing(variables.tabId, true);
      actions.appendLog(variables.tabId, `${variables.action} operation in progress...`);
    },
    onSuccess: (_, variables) => {
      actions.appendLog(variables.tabId, `${variables.action} completed successfully`);
      actions.setProcessing(variables.tabId, false);
      
      if (variables.action === 'delete' || variables.action === 'removeConfig') {
        actions.setInstanceDetails(variables.tabId, null);
        actions.clearLogs(variables.tabId);
        store.consoleMode = { ...store.consoleMode, [variables.tabId]: 'deployment' };
        queryClient.invalidateQueries({ queryKey: ['configs'] });
      } else {
        queryClient.invalidateQueries({ queryKey: ['instanceDetails', variables.tabId] });
        queryClient.invalidateQueries({ queryKey: ['containerLogs', variables.tabId] });
      }
    },
    onError: (error: unknown, variables) => {
      const errorMsg = axios.isAxiosError(error)
        ? error.response?.data?.error || error.message
        : 'Operation failed';
      actions.appendLog(variables.tabId, `Error: ${errorMsg}`);
      actions.setProcessing(variables.tabId, false);
    },
  });

  const exportConfig = useMutation({
    mutationFn: exportConfigFn,
    onSuccess: (data) => {
      const blob = new Blob([data], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = '.env.export';
      a.click();
      URL.revokeObjectURL(url);
    },
    onError: () => {
      alert('Failed to export config. Is NATS running?');
    },
  });

  return {
    testConnection,
    executeDeploy,
    containerAction,
    exportConfig,
  };
};
