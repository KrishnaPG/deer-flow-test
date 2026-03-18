import { useEffect } from 'react';
import { useSnapshot } from 'valtio';
import { store, selectActiveTab, selectIsServiceManaged } from '../store';
import * as actions from '../store/actions';

export const useInstanceDetailsLoader = (): void => {
  const snap = useSnapshot(store);
  const activeTabId = snap.activeTabId;
  const activeTab = selectActiveTab(snap);
  const isHealthy = activeTabId ? !!snap.configs[`${activeTabId}.url`] : false;
  const hasInstanceDetails = !!activeTab?.instanceDetails;
  const isManaged = selectIsServiceManaged(snap, activeTabId);
  const hasLogs = activeTabId ? (snap.logs[activeTabId]?.length ?? 0) > 0 : false;
  const natsUrl = snap.natsUrl;

  useEffect(() => {
    // Only fetch if we have an active tab, it's healthy, and we don't have instance details yet
    if (!activeTabId || !isHealthy || hasInstanceDetails) return;

    const loadDetails = async () => {
      try {
        await actions.loadInstanceDetails(activeTabId, natsUrl);
      } catch (err) {
        console.error('Failed to load instance details:', err);
      }
    };

    loadDetails();
  }, [activeTabId, isHealthy, hasInstanceDetails, natsUrl]);

  useEffect(() => {
    // Load container logs for managed services
    if (!activeTabId || !isHealthy || !isManaged || hasLogs) return;

    const loadLogs = async () => {
      try {
        await actions.loadContainerLogs(activeTabId, natsUrl);
      } catch (err) {
        console.error('Failed to load container logs:', err);
      }
    };

    loadLogs();
  }, [activeTabId, isHealthy, isManaged, hasLogs, natsUrl]);
};
