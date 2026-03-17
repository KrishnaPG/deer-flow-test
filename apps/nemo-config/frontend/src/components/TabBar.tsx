import { X } from 'lucide-react';
import { ServiceStatus } from '../types';

interface Tab {
  id: string;
  name: string;
  status: ServiceStatus;
}

interface TabBarProps {
  tabs: Tab[];
  activeTabId: string | null;
  onTabClick: (id: string) => void;
  onTabClose: (id: string) => void;
}

export function TabBar({ tabs, activeTabId, onTabClick, onTabClose }: TabBarProps) {
  if (tabs.length === 0) {
    return (
      <div className="h-10 border-b border-gray-200 bg-gray-50 flex items-center px-4">
        <span className="text-sm text-gray-400">Select a service from the sidebar to open</span>
      </div>
    );
  }

  return (
    <div className="flex border-b border-gray-200 bg-gray-100 overflow-x-auto">
      {tabs.map(tab => (
        <div
          key={tab.id}
          onClick={() => onTabClick(tab.id)}
          className={`group flex items-center gap-2 px-4 py-2 cursor-pointer border-r border-gray-200 text-sm min-w-fit ${
            activeTabId === tab.id
              ? 'bg-white text-gray-900 border-t-2 border-t-blue-500'
              : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
          }`}
        >
          <StatusDot status={tab.status} />
          <span className="truncate max-w-[150px]">{tab.name}</span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onTabClose(tab.id);
            }}
            className="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-gray-300 rounded transition-all"
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}

function StatusDot({ status }: { status: ServiceStatus }) {
  return (
    <span
      className={`w-2 h-2 rounded-full ${
        status === 'healthy'
          ? 'bg-green-500'
          : status === 'deploying'
          ? 'bg-blue-500 animate-pulse'
          : 'bg-gray-400'
      }`}
    />
  );
}
