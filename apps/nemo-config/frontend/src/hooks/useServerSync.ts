import { useEffect } from 'react';
import { QueryClient, useQuery } from '@tanstack/react-query';
import { store } from '../store';
import * as apiActions from '../store/api-actions';
import {
  fetchTemplates,
  fetchHosts,
  fetchConfigs,
  checkNatsHealth,
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

  // Sync templates to store
  useEffect(() => {
    apiActions.setTemplates(templatesQuery.data || []);
    apiActions.setLoadingTemplates(templatesQuery.isLoading);
    apiActions.setTemplatesError(templatesQuery.error?.message || null);
  }, [templatesQuery.data, templatesQuery.isLoading, templatesQuery.error]);

  // Sync hosts to store
  useEffect(() => {
    if (hostsQuery.data) {
      apiActions.setHosts(hostsQuery.data);
    }
  }, [hostsQuery.data]);

  // Sync configs to store
  useEffect(() => {
    apiActions.setConfigs(configsQuery.data || {});
    apiActions.setLoadingConfigs(configsQuery.isLoading);
    apiActions.setConfigsError(configsQuery.error?.message || null);
  }, [configsQuery.data, configsQuery.isLoading, configsQuery.error]);

  // Sync NATS status to store
  useEffect(() => {
    const status = natsHealthQuery.isLoading ? 'checking' 
      : natsHealthQuery.data ? 'connected' 
      : 'disconnected';
    apiActions.setNatsStatus(status);
  }, [natsHealthQuery.data, natsHealthQuery.isLoading]);
};
