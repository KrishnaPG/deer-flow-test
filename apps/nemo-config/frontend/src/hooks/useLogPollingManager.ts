import { useEffect, useRef } from 'react';
import { useSnapshot } from 'valtio';
import { store, selectIsServiceManaged } from '../store';
import { fetchContainerLogs } from '../api/services';

export const useLogPollingManager = (): void => {
  const snap = useSnapshot(store);
  const logPollingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  
  const activeTabId = snap.activeTabId;
  const isHealthy = activeTabId ? !!snap.configs[`${activeTabId}.url`] : false;
  const isManaged = selectIsServiceManaged(snap, activeTabId);
  const consoleMode = activeTabId ? snap.consoleMode[activeTabId] : 'deployment';
  const isContainerMode = consoleMode === 'container';

  useEffect(() => {
    // Clear existing interval
    if (logPollingRef.current) {
      clearInterval(logPollingRef.current);
      logPollingRef.current = null;
    }

    // Only poll if all conditions are met
    if (activeTabId && isHealthy && isManaged && isContainerMode) {
      const pollLogs = async () => {
        try {
          const logs = await fetchContainerLogs(activeTabId, snap.natsUrl);
          store.logs = { ...store.logs, [activeTabId]: logs };
        } catch (err) {
          console.error('Failed to poll container logs:', err);
        }
      };
      
      // Poll immediately and then every second
      pollLogs();
      logPollingRef.current = setInterval(pollLogs, 1000);
    }

    return () => {
      if (logPollingRef.current) {
        clearInterval(logPollingRef.current);
        logPollingRef.current = null;
      }
    };
  }, [activeTabId, isHealthy, isManaged, isContainerMode, snap.natsUrl]);
};
