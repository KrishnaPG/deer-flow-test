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

  // For external services without full instance details, show minimal info
  if (!activeTab.instanceDetails) {
    const configUrl = snap.configs[`${activeTab.id}.url`];
    return (
      <div className="space-y-4">
        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <h3 className="text-lg font-semibold text-green-800 mb-2">External Service Registered</h3>
          <p className="text-sm text-green-700">
            This external service has been successfully registered.
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
