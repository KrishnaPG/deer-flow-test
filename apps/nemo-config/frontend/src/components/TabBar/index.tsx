import { useSnapshot } from 'valtio';
import { store, selectServiceStatus } from '../../store';
import * as actions from '../../store/actions';
import { X } from 'lucide-react';

export const TabBar = () => {
  const snap = useSnapshot(store);
  
  if (snap.tabs.length === 0) return null;
  
  return (
    <div className="flex items-center bg-gray-50 border-b border-gray-200 overflow-x-auto">
      {snap.tabs.map(tab => {
        const isActive = tab.id === snap.activeTabId;
        const status = selectServiceStatus(snap, tab.id);
        
        return (
          <div
            key={tab.id}
            onClick={() => actions.setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-4 py-3 border-r border-gray-200 cursor-pointer transition-colors min-w-fit ${
              isActive
                ? 'bg-white border-t-2 border-t-blue-500'
                : 'bg-gray-50 hover:bg-gray-100'
            }`}
          >
            <span className="text-sm font-medium text-gray-700">{tab.name}</span>
            
            <span
              className={`w-2 h-2 rounded-full ${
                status === 'healthy'
                  ? 'bg-green-500'
                  : status === 'deploying'
                  ? 'bg-blue-500 animate-pulse'
                  : 'bg-gray-400'
              }`}
            />
            
            <button
              onClick={e => {
                e.stopPropagation();
                actions.closeTab(tab.id);
              }}
              className="p-0.5 hover:bg-gray-200 rounded opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <X size={14} className="text-gray-500" />
            </button>
          </div>
        );
      })}
    </div>
  );
};
