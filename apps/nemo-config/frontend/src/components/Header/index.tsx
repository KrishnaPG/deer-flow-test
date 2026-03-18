import { useSnapshot } from 'valtio';
import { store } from '../../store';
import * as actions from '../../store/actions';
import * as apiActions from '../../store/api-actions';
import { Settings, Activity, CheckCircle, XCircle, Download } from 'lucide-react';

export const Header = () => {
  const snap = useSnapshot(store);
  
  const getStatusIcon = () => {
    switch (snap.natsStatus) {
      case 'checking':
        return <Activity size={16} className="text-yellow-500 animate-pulse" />;
      case 'connected':
        return <CheckCircle size={16} className="text-green-500" />;
      default:
        return <XCircle size={16} className="text-red-500" />;
    }
  };
  
  const getStatusText = () => {
    switch (snap.natsStatus) {
      case 'checking':
        return 'Checking...';
      case 'connected':
        return 'Connected';
      default:
        return 'Disconnected';
    }
  };
  
  return (
    <div className="bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between">
      <div className="flex items-center gap-3">
        <h1 className="text-xl font-bold text-gray-900">Nemo Config</h1>
        <span className="text-sm text-gray-500">{snap.natsUrl}</span>
      </div>
      
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 px-3 py-1.5 bg-gray-100 rounded-full">
          {getStatusIcon()}
          <span className="text-sm text-gray-700">{getStatusText()}</span>
        </div>
        
        <button
          onClick={() => actions.openSettings()}
          className="p-2 hover:bg-gray-100 rounded-md transition-colors"
          title="Settings"
        >
          <Settings size={20} className="text-gray-600" />
        </button>
        
        <button
          onClick={() => apiActions.exportConfig()}
          className="flex items-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-sm font-medium transition-colors"
        >
          <Download size={16} />
          Export
        </button>
      </div>
    </div>
  );
};
