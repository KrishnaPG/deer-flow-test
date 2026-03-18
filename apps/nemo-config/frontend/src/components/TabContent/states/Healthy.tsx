import { useSnapshot } from 'valtio';
import { store, selectActiveTab } from '../../../store';
import * as apiActions from '../../../store/api-actions';
import { InstanceDetailsView } from '../../InstanceDetails';

export const TabHealthy = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  
  if (!activeTab?.instanceDetails) return null;
  
  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
  };
  
  const handleStop = () => {
    if (snap.activeTabId) {
      apiActions.handleContainerAction(snap.activeTabId, 'stop');
    }
  };
  
  const handleStart = () => {
    if (snap.activeTabId) {
      apiActions.handleContainerAction(snap.activeTabId, 'start');
    }
  };
  
  const handleRestart = () => {
    if (snap.activeTabId) {
      apiActions.handleContainerAction(snap.activeTabId, 'restart');
    }
  };
  
  const handleDelete = () => {
    if (snap.activeTabId) {
      apiActions.handleContainerAction(snap.activeTabId, 'delete');
    }
  };
  
  const handleRemoveConfig = () => {
    if (snap.activeTabId) {
      apiActions.handleContainerAction(snap.activeTabId, 'removeConfig');
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
