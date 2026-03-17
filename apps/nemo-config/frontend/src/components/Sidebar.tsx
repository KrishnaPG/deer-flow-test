import { CheckCircle, Activity } from 'lucide-react';
import type { Template, ServiceStatus } from '../definitions';
import { ICONS } from '../definitions';

interface SidebarProps {
  templates: Template[];
  selectedId: string | null;
  status: Record<string, ServiceStatus>;
  onSelect: (id: string) => void;
}

export function Sidebar({ templates, selectedId, status, onSelect }: SidebarProps) {
  const sortedTemplates = [...templates].sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="w-72 bg-gray-50 border-r border-gray-200 flex flex-col h-full">
      <div className="px-4 py-3 border-b border-gray-200 bg-gray-100">
        <h2 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">Services</h2>
      </div>
      <div className="flex-1 overflow-y-auto">
        {sortedTemplates.map(tpl => (
          <ServiceItem
            key={tpl.id}
            template={tpl}
            isSelected={selectedId === tpl.id}
            status={status[tpl.id] || 'unconfigured'}
            onClick={() => onSelect(tpl.id)}
          />
        ))}
      </div>
    </div>
  );
}

interface ServiceItemProps {
  template: Template;
  isSelected: boolean;
  status: ServiceStatus;
  onClick: () => void;
}

function ServiceItem({ template, isSelected, status, onClick }: ServiceItemProps) {
  const Icon = ICONS[template.icon] || ICONS['box'];
  const isUnconfigured = status === 'unconfigured';

  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-3 px-4 py-3 border-b border-gray-200 transition-colors text-left ${isSelected ? 'bg-blue-50 border-l-4 border-l-blue-500' :
          isUnconfigured ? 'hover:bg-gray-100 opacity-60' : 'hover:bg-gray-100'
        }`}
    >
      <div className={`p-1.5 rounded ${status === 'healthy' ? 'bg-green-100 text-green-600' :
          status === 'deploying' ? 'bg-blue-100 text-blue-600' :
            'bg-gray-200 text-gray-500'
        }`}>
        <Icon size={18} />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className={`font-medium text-sm truncate ${isUnconfigured ? 'text-gray-600' : 'text-gray-900'}`}>
            {template.name}
          </span>
          {status === 'healthy' && <CheckCircle size={14} className="text-green-500 flex-shrink-0" />}
          {status === 'deploying' && <Activity size={14} className="text-blue-500 animate-pulse flex-shrink-0" />}
        </div>
        <p className="text-xs text-gray-500 font-mono">Port: {template.default_port}</p>
      </div>
    </button>
  );
}
