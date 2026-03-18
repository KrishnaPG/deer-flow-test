import { useEffect, useRef } from 'react';
import { fetchContainerLogs } from '../api/services';

interface Tab {
  id: string;
  consoleMode: 'deployment' | 'container';
  instanceDetails: { metadata?: { managedBy?: 'nemo' | 'external' } } | null;
}

export function useLogPolling(
  activeTabId: string | null,
  tabs: Tab[],
  status: Record<string, string>,
  natsUrl: string,
  onLogsUpdate: (tabId: string, logs: string[]) => void
) {
  const logPollingRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (!activeTabId) {
      if (logPollingRef.current) {
        clearInterval(logPollingRef.current);
        logPollingRef.current = null;
      }
      return;
    }

    const activeTab = tabs.find(t => t.id === activeTabId);
    if (!activeTab) return;

    const isHealthy = status[activeTabId] === 'healthy';
    const isManaged = activeTab.instanceDetails?.metadata?.managedBy === 'nemo';
    const isContainerMode = activeTab.consoleMode === 'container';

    if (isHealthy && isManaged && isContainerMode) {
      const pollLogs = async () => {
        const logs = await fetchContainerLogs(activeTabId, natsUrl);
        onLogsUpdate(activeTabId, logs);
      };
      
      pollLogs();
      logPollingRef.current = setInterval(pollLogs, 1000);

      return () => {
        if (logPollingRef.current) {
          clearInterval(logPollingRef.current);
          logPollingRef.current = null;
        }
      };
    } else {
      if (logPollingRef.current) {
        clearInterval(logPollingRef.current);
        logPollingRef.current = null;
      }
    }
  }, [activeTabId, tabs, status, natsUrl, onLogsUpdate]);

  return logPollingRef;
}