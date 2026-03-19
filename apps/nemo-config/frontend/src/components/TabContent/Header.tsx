import { useSnapshot } from 'valtio';
import { store, selectActiveTemplate } from '../../store';
import * as actions from '../../store/actions';
import { Server, Link2 } from 'lucide-react';

export const TabHeader = () => {
  const snap = useSnapshot(store);
  const template = selectActiveTemplate(snap);
  const isHealthy = !!snap.configs[`nemo.${snap.activeTabId}.url`];
  
  if (!template) return null;
  
  const handleModeChange = (mode: 'deploy' | 'existing') => {
    if (snap.activeTabId) {
      actions.setTabMode(snap.activeTabId, mode);
    }
  };
  
  return (
    <div className="px-6 py-3 border-b border-gray-200 flex items-center justify-between bg-white">
      <div>
        <h2 className="text-lg font-semibold text-gray-900">{template.name}</h2>
        <p className="text-xs text-gray-500 font-mono">Port: {template.default_port}</p>
      </div>
      {!isHealthy && (
        <div className="bg-gray-100 p-1 rounded-lg inline-flex">
          <button
            onClick={() => handleModeChange('deploy')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all ${
              snap.tabs.find(t => t.id === snap.activeTabId)?.mode === 'deploy'
                ? 'bg-white text-gray-900 shadow-sm'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <Server size={14} /> Deploy New
          </button>
          <button
            onClick={() => handleModeChange('existing')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all ${
              snap.tabs.find(t => t.id === snap.activeTabId)?.mode === 'existing'
                ? 'bg-white text-gray-900 shadow-sm'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <Link2 size={14} /> Use Existing
          </button>
        </div>
      )}
    </div>
  );
};
