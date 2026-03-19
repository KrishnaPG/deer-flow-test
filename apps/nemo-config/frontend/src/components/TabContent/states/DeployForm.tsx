import { useState, useEffect, useCallback } from 'react';
import { useSnapshot } from 'valtio';
import { store, selectActiveTab, selectActiveTemplate } from '../../../store';
import * as actions from '../../../store/actions';

export const TabDeployForm = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  const template = selectActiveTemplate(snap);
  const isDeploying = snap.deploying === snap.activeTabId;

  // Local state for form values to prevent cursor jumping
  const [localValues, setLocalValues] = useState<Record<string, string>>({});

  // Sync local values with store when tab changes
  useEffect(() => {
    if (activeTab) {
      setLocalValues(activeTab.formValues);
    }
  }, [activeTab?.id]);

  if (!activeTab || !template) return null;

  const handleHostChange = useCallback((host: string) => {
    if (snap.activeTabId) {
      actions.setSelectedHost(snap.activeTabId, host);
    }
  }, [snap.activeTabId]);

  const handleInputChange = useCallback((key: string, value: string) => {
    // Update local state immediately (no cursor jump)
    setLocalValues(prev => ({ ...prev, [key]: value }));
  }, []);

  const handleInputBlur = useCallback((key: string, value: string) => {
    // Sync to store on blur
    if (snap.activeTabId) {
      actions.setFormValue(snap.activeTabId, key, value);
    }
  }, [snap.activeTabId]);

  return (
    <>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1.5">Target Host</label>
        <select
          className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none"
          value={activeTab.selectedHost}
          onChange={e => handleHostChange(e.target.value)}
          disabled={isDeploying}
        >
          {snap.hosts.map(h => <option key={h} value={h}>{h}</option>)}
        </select>
        <p className="text-xs text-gray-500 mt-1">SSH host to deploy Docker container</p>
      </div>

      {template.env_vars.map((env: { key: string; description: string; secret?: boolean; default?: string }) => {
        const isDataPath = env.key === 'DATA_PATH';

        return (
          <div key={env.key}>
            <label className="block text-sm font-medium text-gray-700 mb-1.5">
              {env.description || env.key}
            </label>
            <input
              type={env.secret ? 'password' : 'text'}
              className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
              value={localValues[env.key] ?? activeTab.formValues[env.key] ?? ''}
              onChange={e => handleInputChange(env.key, e.target.value)}
              onBlur={e => handleInputBlur(env.key, e.target.value)}
              disabled={isDeploying}
              placeholder={env.default ? `Default: ${env.default}` : `Enter ${env.key}`}
            />
            {isDataPath && (
              <p className="text-xs text-gray-500 mt-1">
                Docker volume mount on the host, either absolute path or relative to the 'Deploy Path' of the service
              </p>
            )}
          </div>
        );
      })}
    </>
  );
};
