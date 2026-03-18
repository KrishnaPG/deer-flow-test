import { useSnapshot } from 'valtio';
import { store, selectActiveTab, selectActiveTemplate } from '../../../store';
import * as actions from '../../../store/actions';

export const TabDeployForm = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  const template = selectActiveTemplate(snap);
  const isDeploying = snap.deploying === snap.activeTabId;
  
  if (!activeTab || !template) return null;
  
  const handleHostChange = (host: string) => {
    if (snap.activeTabId) {
      actions.setSelectedHost(snap.activeTabId, host);
    }
  };
  
  const handleFormChange = (key: string, value: string) => {
    if (snap.activeTabId) {
      actions.setFormValue(snap.activeTabId, key, value);
    }
  };
  
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

      {template.env_vars.map((env: { key: string; description: string; secret?: boolean; default?: string }) => (
        <div key={env.key}>
          <label className="block text-sm font-medium text-gray-700 mb-1.5">
            {env.description || env.key}
          </label>
          <input
            type={env.secret ? 'password' : 'text'}
            className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
            value={activeTab.formValues[env.key] || ''}
            onChange={e => handleFormChange(env.key, e.target.value)}
            disabled={isDeploying}
            placeholder={env.default ? `Default: ${env.default}` : `Enter ${env.key}`}
          />
        </div>
      ))}
    </>
  );
};
