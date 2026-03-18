import React from 'react';
import { useSnapshot } from 'valtio';
import { Activity, Check, AlertCircle } from 'lucide-react';
import { store, selectActiveTab, selectActiveTemplate } from '../../../store';
import * as actions from '../../../store/actions';
import { useMutations } from '../../../hooks/useMutations';

export const TabExistingForm = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  const template = selectActiveTemplate(snap);
  const isDeploying = snap.deploying === snap.activeTabId;
  const { testConnection } = useMutations();
  
  if (!activeTab || !template) return null;
  
  const { testStatus, testMessage, existingUrl, formValues } = activeTab;
  const isTesting = testStatus === 'testing';
  const hasTested = testStatus === 'success' || testStatus === 'error';
  const isSuccess = testStatus === 'success';
  
  const handleExistingUrlChange = (url: string) => {
    if (snap.activeTabId) {
      actions.setExistingUrl(snap.activeTabId, url);
    }
  };
  
  const handleFormChange = (key: string, value: string) => {
    if (snap.activeTabId) {
      actions.setFormValue(snap.activeTabId, key, value);
    }
  };
  
  const handleTestConnection = () => {
    if (snap.activeTabId && template) {
      if (!activeTab?.existingUrl.trim()) {
        actions.setTestStatus(snap.activeTabId, 'error', 'Please enter a connection URL');
        actions.appendLog(snap.activeTabId, 'Error: No connection URL provided');
        return;
      }
      testConnection.mutate({
        service_id: snap.activeTabId,
        connection_url: activeTab.existingUrl,
        health_check: template.health_check,
        metadata: activeTab.formValues,
      });
    }
  };
  
  const hasSecretVars = template.env_vars.some((e: { secret?: boolean }) => e.secret);
  
  return (
    <>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1.5">Connection URL</label>
        <input
          type="text"
          className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
          value={existingUrl}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleExistingUrlChange(e.target.value)}
          disabled={isDeploying}
          placeholder={template.connection_url_pattern || 'e.g., postgres://user:pass@host:5432/db'}
        />
        <p className="text-xs text-gray-500 mt-1">Full connection string to existing {template.name} instance</p>
      </div>

      {hasSecretVars && (
        <div className="bg-amber-50 border border-amber-200 rounded-md p-4">
          <h4 className="text-sm font-medium text-amber-900 mb-2">Optional Credentials</h4>
          <div className="grid grid-cols-2 gap-3">
            {template.env_vars.filter((e: { secret?: boolean }) => e.secret).map((env: { key: string; description: string }) => (
              <div key={env.key}>
                <label className="block text-xs font-medium text-gray-700 mb-1">
                  {env.description || env.key}
                </label>
                <input
                  type="password"
                  className="w-full border-gray-300 rounded-md shadow-sm p-2 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-xs"
                  value={formValues[env.key] || ''}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleFormChange(env.key, e.target.value)}
                  disabled={isDeploying}
                />
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex items-center gap-3">
        <button
          onClick={handleTestConnection}
          disabled={isDeploying || isTesting || !existingUrl.trim()}
          className={`flex items-center gap-2 px-4 py-2 rounded-md shadow-sm text-sm font-medium transition-colors ${
            isSuccess 
              ? 'bg-green-100 text-green-700 hover:bg-green-200 border border-green-300' 
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200 border border-gray-300'
          } disabled:opacity-50 disabled:cursor-not-allowed`}
        >
          {isTesting ? (
            <><Activity size={16} className="animate-spin" /> Testing...</>
          ) : isSuccess ? (
            <><Check size={16} /> Connection Verified</>
          ) : (
            <>Test Connection</>
          )}
        </button>

        {hasTested && testMessage && (
          <div className={`flex items-center gap-1.5 text-sm ${isSuccess ? 'text-green-600' : 'text-red-600'}`}>
            {isSuccess ? <Check size={16} /> : <AlertCircle size={16} />}
            <span>{testMessage}</span>
          </div>
        )}
      </div>

      {!isSuccess && (
        <div className="bg-blue-50 border border-blue-200 rounded-md p-3">
          <p className="text-xs text-blue-700">
            You must test the connection successfully before registering the existing instance.
          </p>
        </div>
      )}
    </>
  );
};
