import { useEffect } from 'react';
import { QueryClient, useQuery } from '@tanstack/react-query';
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
    queryKey: ['configs', store.natsUrl],
    queryFn: () => fetchConfigs(store.natsUrl),
    enabled: !!store.natsUrl,
  });

  // NATS health query
  const natsHealthQuery = useQuery({
    queryKey: ['natsHealth', store.natsUrl],
    queryFn: () => checkNatsHealth(store.natsUrl),
    refetchInterval: 10000,
    enabled: !!store.natsUrl,
  });

  // Instance details query - fetch for active tab
  const instanceDetailsQuery = useQuery({
    queryKey: ['instanceDetails', store.activeTabId],
    queryFn: () => fetchInstanceDetails(store.activeTabId!, store.natsUrl),
    enabled: !!store.activeTabId && !!store.natsUrl && !!store.configs[`${store.activeTabId}.url`],
  });

  // Container logs query - only for managed services in container mode
  const containerLogsQuery = useQuery({
    queryKey: ['containerLogs', store.activeTabId],
    queryFn: () => fetchContainerLogs(store.activeTabId!, store.natsUrl),
    refetchInterval: 1000,
    enabled: !!store.activeTabId && 
              !!store.natsUrl && 
              !!store.configs[`${store.activeTabId}.url`] &&
              store.consoleMode[store.activeTabId] === 'container',
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
    if (store.activeTabId && instanceDetailsQuery.data) {
      actions.setInstanceDetails(store.activeTabId, instanceDetailsQuery.data);
    }
  }, [instanceDetailsQuery.data, store.activeTabId]);

  // Sync container logs to store
  useEffect(() => {
    if (store.activeTabId && containerLogsQuery.data) {
      store.logs = { ...store.logs, [store.activeTabId]: containerLogsQuery.data };
    }
  }, [containerLogsQuery.data, store.activeTabId]);
};
