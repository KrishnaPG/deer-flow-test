import { useSnapshot } from 'valtio';
import { store } from '../../store';
import * as actions from '../../store/actions';
import { X } from 'lucide-react';

// Type definition for TabContent props to avoid conflicts
export interface TabContentProps {
  template: any;
  status: string;
  mode: string;
  formValues: Record<string, string>;
  selectedHost: string;
  existingUrl: string;
  hosts: string[];
  consoleOutput: string[];
  consoleMode: string;
  isDeploying: boolean;
  testStatus: string;
  testMessage?: string;
  instanceDetails: any;
  onModeChange: (mode: string) => void;
  onFormChange: (key: string, value: string) => void;
  onHostChange: (host: string) => void;
  onExistingUrlChange: (url: string) => void;
  onTestConnection: () => void;
  onDeploy: () => void;
  onCopy: (text: string) => void;
  onStop: () => void;
  onStart: () => void;
  onRestart: () => void;
  onDelete: () => void;
  onRemoveConfig: () => void;
  isProcessing: boolean;
}

export const Settings = () => {
  const snap = useSnapshot(store, { sync: true });
  
  if (!snap.showSettings) return null;
  
  const handleSave = () => {
    actions.saveSettings();
  };
  
  const handleCancel = () => {
    actions.closeSettings();
  };
  
  return (
    <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Settings</h2>
          <button
            onClick={handleCancel}
            className="p-1 hover:bg-gray-100 rounded-md transition-colors"
          >
            <X size={20} className="text-gray-500" />
          </button>
        </div>
        
        <div className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1.5">
              CONSUL URL
            </label>
            <input
              type="text"
              className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
              value={snap.settingsInputs.consulUrl}
              onChange={e => actions.updateSettingsInput('consulUrl', e.target.value)}
              placeholder="http://localhost:8500"
            />
            <p className="text-xs text-gray-500 mt-1">
              Connection string for CONSUL server
            </p>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1.5">
              Deploy Path
            </label>
            <input
              type="text"
              className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
              value={snap.settingsInputs.deployPath}
              onChange={e => actions.updateSettingsInput('deployPath', e.target.value)}
              placeholder="~/workspace/nemo"
            />
            <p className="text-xs text-gray-500 mt-1">
              Base path for Docker deployments on remote hosts
            </p>
          </div>
        </div>
        
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200 bg-gray-50 rounded-b-lg">
          <button
            onClick={handleCancel}
            className="px-4 py-2 text-gray-700 hover:bg-gray-200 rounded-md text-sm font-medium transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-sm font-medium transition-colors"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
};
