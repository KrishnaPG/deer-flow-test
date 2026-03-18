import { useSnapshot } from 'valtio';
import { store, selectActiveTab } from '../../../store';
import { useMutations } from '../../../hooks/useMutations';
import { InstanceDetailsView } from '../../InstanceDetails';

export const TabHealthy = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  const { containerAction } = useMutations();

  if (!activeTab?.instanceDetails) return null;

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const handleStop = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'stop',
        natsUrl: snap.natsUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleStart = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'start',
        natsUrl: snap.natsUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleRestart = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'restart',
        natsUrl: snap.natsUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleDelete = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'delete',
        natsUrl: snap.natsUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleRemoveConfig = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'removeConfig',
        natsUrl: snap.natsUrl,
        deployPath: snap.deployPath,
      });
    }
  };
  
  return (
    <InstanceDetailsView
      instanceDetails={activeTab.instanceDetails}
      onCopy={handleCopy}
      onStop={handleStop}
      onStart={handleStart}
      onRestart={handleRestart}
      onDelete={handleDelete}
      onRemoveConfig={handleRemoveConfig}
      isProcessing={activeTab.isProcessing}
    />
  );
};
