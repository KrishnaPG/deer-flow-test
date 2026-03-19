import { useEffect } from 'react';
import { QueryClient, useQuery } from '@tanstack/react-query';
import { useSnapshot } from 'valtio';
import { store } from '../store';
import * as actions from '../store/actions';
import {
  fetchTemplates,
  fetchHosts,
  fetchConfigs,
  checkConsulHealth,
  fetchInstanceDetails,
  fetchContainerLogs,
} from '../api/services';
import { CONSUL_PREFIX } from '../../../schema';

const getServiceUrlKey = (serviceId: string) => `${CONSUL_PREFIX}.${serviceId}.url`;

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 3,
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
    },
  },
});

export const retryConsulHealth = async (consulUrl: string): Promise<void> => {
  try {
    const result = await queryClient.fetchQuery({
      queryKey: ['consulHealth', consulUrl],
      queryFn: () => checkConsulHealth(consulUrl),
      retry: false,
    });
    store.consulStatus = result.connected ? 'connected' : 'disconnected';
    store.consulError = result.error || null;
  } catch {
    store.consulStatus = 'disconnected';
    store.consulError = 'Failed to check health';
  }
  store.retryingConnection = false;
};

export const useServerSync = (): void => {
  // Use snapshot to track reactive values from Valtio store
  const snap = useSnapshot(store);
  const { activeTabId, consulUrl, configs, consoleMode } = snap;

  // Templates query
  const templatesQuery = useQuery({
    queryKey: ['templates'],
    queryFn: fetchTemplates,
  });

  // Hosts query - fetch once
  const hostsQuery = useQuery({
    queryKey: ['hosts'],
    queryFn: fetchHosts,
    staleTime: Infinity,
  });

  // Configs query
  const configsQuery = useQuery({
    queryKey: ['configs', consulUrl],
    queryFn: () => fetchConfigs(consulUrl),
    enabled: !!consulUrl,
  });

  // CONSUL health query
  const consulHealthQuery = useQuery({
    queryKey: ['consulHealth', consulUrl],
    queryFn: () => checkConsulHealth(consulUrl),
    refetchInterval: 10000,
    enabled: !!consulUrl,
  });

  // Instance details query - fetch for active tab
  const instanceDetailsQuery = useQuery({
    queryKey: ['instanceDetails', activeTabId],
    queryFn: () => fetchInstanceDetails(activeTabId!, consulUrl),
    enabled: !!activeTabId && !!consulUrl && !!configs[getServiceUrlKey(activeTabId)],
  });

  // Container logs query - only for managed services in container mode
  const containerLogsQuery = useQuery({
    queryKey: ['containerLogs', activeTabId],
    queryFn: () => fetchContainerLogs(activeTabId!, consulUrl),
    refetchInterval: 1000,
    enabled: !!activeTabId && 
              !!consulUrl && 
              !!configs[getServiceUrlKey(activeTabId)] &&
              consoleMode[activeTabId] === 'container',
  });

  // Sync templates to store
  useEffect(() => {
    store.templates = templatesQuery.data || [];
    store.isLoadingTemplates = templatesQuery.isLoading;
    store.templatesError = templatesQuery.error?.message || null;
  }, [templatesQuery.data, templatesQuery.isLoading, templatesQuery.error]);

  // Sync hosts to store
  useEffect(() => {
    if (hostsQuery.data) {
      store.hosts = hostsQuery.data;
    }
  }, [hostsQuery.data]);

  // Sync configs to store
  useEffect(() => {
    store.configs = configsQuery.data || {};
    store.isLoadingConfigs = configsQuery.isLoading;
    store.configsError = configsQuery.error?.message || null;
  }, [configsQuery.data, configsQuery.isLoading, configsQuery.error]);

  // Sync CONSUL status to store
  useEffect(() => {
    const status = consulHealthQuery.isLoading ? 'checking' 
      : consulHealthQuery.data?.connected ? 'connected' 
      : 'disconnected';
    store.consulStatus = status;
    if (consulHealthQuery.data?.error !== undefined) {
      store.consulError = consulHealthQuery.data.error || null;
    }
    if (status !== 'checking' && store.retryingConnection) {
      store.retryingConnection = false;
    }
  }, [consulHealthQuery.data, consulHealthQuery.isLoading]);

  // Sync instance details to store
  useEffect(() => {
    if (activeTabId && instanceDetailsQuery.data) {
      actions.setInstanceDetails(activeTabId, instanceDetailsQuery.data);
      // If this is a managed service, switch to container logs mode
      if (instanceDetailsQuery.data.metadata?.managedBy === 'nemo') {
        actions.setConsoleMode(activeTabId, 'container');
      }
    }
  }, [instanceDetailsQuery.data, activeTabId]);

  // Sync container logs to store
  useEffect(() => {
    if (activeTabId && containerLogsQuery.data) {
      store.logs = { ...store.logs, [activeTabId]: containerLogsQuery.data };
      // Ensure console mode is set to container when we have container logs
      if (store.consoleMode[activeTabId] !== 'container') {
        actions.setConsoleMode(activeTabId, 'container');
      }
    }
  }, [containerLogsQuery.data, activeTabId]);
};
