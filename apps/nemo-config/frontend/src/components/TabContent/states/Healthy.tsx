import { useSnapshot } from 'valtio';
import { store, selectActiveTab } from '../../../store';
import { useMutations } from '../../../hooks/useMutations';
import { InstanceDetailsView } from '../../InstanceDetails';

export const TabHealthy = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  const { containerAction } = useMutations();

  if (!activeTab) return null;

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const handleStop = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'stop',
        consulUrl: snap.consulUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleStart = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'start',
        consulUrl: snap.consulUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleRestart = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'restart',
        consulUrl: snap.consulUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleDelete = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'delete',
        consulUrl: snap.consulUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const handleRemoveConfig = () => {
    if (snap.activeTabId) {
      containerAction.mutate({
        tabId: snap.activeTabId,
        action: 'removeConfig',
        consulUrl: snap.consulUrl,
        deployPath: snap.deployPath,
      });
    }
  };

  const isExternal = activeTab.instanceDetails?.metadata?.managedBy === 'external';
  
  // For external services (registered via "Use Existing"), show minimal info
  if (isExternal) {
    const configUrl = snap.configs[`nemo.${activeTab.id}.url`];
    return (
      <div className="space-y-4">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h3 className="text-lg font-semibold text-blue-800 mb-2">External Service</h3>
          <p className="text-sm text-blue-700">
            This external service is registered but not managed by Nemo.
          </p>
        </div>
        {configUrl && (
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">Connection URL</label>
            <div className="flex items-center gap-2">
              <code className="flex-1 bg-gray-100 px-3 py-2 rounded text-sm font-mono break-all">
                {configUrl}
              </code>
              <button
                onClick={() => handleCopy(configUrl)}
                className="px-3 py-2 bg-white border border-gray-300 rounded-md text-sm hover:bg-gray-50"
              >
                Copy
              </button>
            </div>
          </div>
        )}
        <div className="flex gap-2">
          <button
            onClick={handleRemoveConfig}
            disabled={activeTab.isProcessing}
            className="px-4 py-2 bg-red-600 text-white rounded-md text-sm hover:bg-red-700 disabled:opacity-50"
          >
            Remove Configuration
          </button>
        </div>
      </div>
    );
  }
  
  // If instance details not yet loaded for a managed service, show loading
  if (!activeTab.instanceDetails) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading service details...</div>
      </div>
    );
  }

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
