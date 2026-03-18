import { useEffect } from 'react';
import { QueryClient, useQuery } from '@tanstack/react-query';
import { useSnapshot } from 'valtio';
import { store } from '../store';
import * as actions from '../store/actions';
import {
  fetchTemplates,
  fetchHosts,
  fetchConfigs,
  checkNatsHealth,
  fetchInstanceDetails,
  fetchContainerLogs,
} from '../api/services';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 3,
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
    },
  },
});

export const useServerSync = (): void => {
  // Use snapshot to track reactive values from Valtio store
  const snap = useSnapshot(store);
  const { activeTabId, natsUrl, configs, consoleMode } = snap;

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
    queryKey: ['configs', natsUrl],
    queryFn: () => fetchConfigs(natsUrl),
    enabled: !!natsUrl,
  });

  // NATS health query
  const natsHealthQuery = useQuery({
    queryKey: ['natsHealth', natsUrl],
    queryFn: () => checkNatsHealth(natsUrl),
    refetchInterval: 10000,
    enabled: !!natsUrl,
  });

  // Instance details query - fetch for active tab
  const instanceDetailsQuery = useQuery({
    queryKey: ['instanceDetails', activeTabId],
    queryFn: () => fetchInstanceDetails(activeTabId!, natsUrl),
    enabled: !!activeTabId && !!natsUrl && !!configs[`${activeTabId}.url`],
  });

  // Container logs query - only for managed services in container mode
  const containerLogsQuery = useQuery({
    queryKey: ['containerLogs', activeTabId],
    queryFn: () => fetchContainerLogs(activeTabId!, natsUrl),
    refetchInterval: 1000,
    enabled: !!activeTabId && 
              !!natsUrl && 
              !!configs[`${activeTabId}.url`] &&
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

  // Sync NATS status to store
  useEffect(() => {
    const status = natsHealthQuery.isLoading ? 'checking' 
      : natsHealthQuery.data ? 'connected' 
      : 'disconnected';
    store.natsStatus = status;
  }, [natsHealthQuery.data, natsHealthQuery.isLoading]);

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
