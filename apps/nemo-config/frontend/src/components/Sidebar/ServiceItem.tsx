import { useSnapshot } from 'valtio';
import { CheckCircle, Activity, Loader2 } from 'lucide-react';
import { store, selectServiceStatus } from '../../store';
import { ICONS } from '../../definitions';
import * as actions from '../../store/actions';
import type { Template } from '../../definitions';

interface ServiceItemProps {
  templateId: string;
}

export const ServiceItem = ({ templateId }: ServiceItemProps) => {
  const snap = useSnapshot(store);
  const template = snap.templates.find(t => t.id === templateId);
  const status = selectServiceStatus(snap, templateId);
  const isSelected = snap.activeTabId === templateId;
  const showCheckingSpinner = snap.isLoadingConfigs && status === 'unconfigured';
  
  if (!template) return null;
  
  const Icon = ICONS[template.icon] || ICONS['box'];
  const isUnconfigured = status === 'unconfigured';
  
  const handleClick = () => {
    actions.openTab(template as Template);
  };
  
  return (
    <button
      onClick={handleClick}
      className={`w-full flex items-center gap-3 px-4 py-3 border-b border-gray-200 transition-colors text-left ${
        isSelected 
          ? 'bg-blue-50 border-l-4 border-l-blue-500' 
          : isUnconfigured 
            ? 'hover:bg-gray-100 opacity-60' 
            : 'hover:bg-gray-100'
      }`}
    >
      <div className={`p-1.5 rounded ${
        status === 'healthy' ? 'bg-green-100 text-green-600' :
        status === 'deploying' ? 'bg-blue-100 text-blue-600' :
        'bg-gray-200 text-gray-500'
      }`}>
        <Icon size={18} />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className={`font-medium text-sm truncate ${
            isUnconfigured ? 'text-gray-600' : 'text-gray-900'
          }`}>
            {template.name}
          </span>
          {status === 'healthy' && <CheckCircle size={14} className="text-green-500 flex-shrink-0" />}
          {status === 'deploying' && <Activity size={14} className="text-blue-500 animate-pulse flex-shrink-0" />}
          {showCheckingSpinner && <Loader2 size={14} className="text-yellow-500 animate-spin flex-shrink-0" />}
        </div>
        <p className="text-xs text-gray-500 font-mono">Port: {template.default_port}</p>
      </div>
    </button>
  );
};
