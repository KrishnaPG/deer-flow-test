import { useSnapshot } from 'valtio';
import { store, selectActiveTab } from '../../../store';
import { useMutations } from '../../../hooks/useMutations';
import { InstanceDetailsView } from '../../InstanceDetails';
import { Copy, Trash2, ExternalLink } from 'lucide-react';

export const TabHealthy = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  const { containerAction } = useMutations();

  if (!activeTab) return null;

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
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
  const configUrl = snap.configs[`nemo.${activeTab.id}.url`];

  if (isExternal) {
    return (
      <div className="space-y-4">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-2">
            <ExternalLink size={20} className="text-blue-600" />
            <h3 className="text-lg font-semibold text-blue-800">External Service</h3>
          </div>
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
                <Copy size={14} />
              </button>
            </div>
          </div>
        )}
        <div className="flex gap-2">
          <button
            onClick={handleRemoveConfig}
            disabled={activeTab.isProcessing}
            className="inline-flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Trash2 size={14} />
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
      onStop={(id) => containerAction.mutate({ tabId: id, action: 'stop', consulUrl: snap.consulUrl, deployPath: snap.deployPath })}
      onStart={(id) => containerAction.mutate({ tabId: id, action: 'start', consulUrl: snap.consulUrl, deployPath: snap.deployPath })}
      onRestart={(id) => containerAction.mutate({ tabId: id, action: 'restart', consulUrl: snap.consulUrl, deployPath: snap.deployPath })}
      onDelete={(id) => containerAction.mutate({ tabId: id, action: 'delete', consulUrl: snap.consulUrl, deployPath: snap.deployPath })}
      onRemoveConfig={handleRemoveConfig}
      isProcessing={activeTab.isProcessing}
    />
  );
};
