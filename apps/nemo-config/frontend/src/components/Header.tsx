import { Download, Settings } from 'lucide-react';

type NatsStatus = 'connected' | 'disconnected' | 'checking';

interface HeaderProps {
  natsUrl: string;
  natsStatus: NatsStatus;
  onSettingsClick: () => void;
  onExport: () => void;
}

export function Header({ natsUrl, natsStatus, onSettingsClick, onExport }: HeaderProps) {
  return (
    <div className="flex justify-between items-center px-6 py-4 bg-white border-b border-gray-200">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">🦑 Nemo-Config</h1>
        <p className="text-sm text-gray-500">Infrastructure Control Plane</p>
      </div>
      <div className="flex items-center gap-3">
        <NatsIndicator url={natsUrl} status={natsStatus} />
        <button 
          onClick={onSettingsClick}
          className="flex items-center gap-2 bg-gray-200 text-gray-700 px-3 py-2 rounded-md hover:bg-gray-300 transition-colors"
          title="Configure NATS URL"
        >
          <Settings size={18} />
        </button>
        <button 
          onClick={onExport}
          className="flex items-center gap-2 bg-gray-800 text-white px-4 py-2 rounded-md hover:bg-gray-700 transition-colors shadow-sm"
        >
          <Download size={18} /> Export .env
        </button>
      </div>
    </div>
  );
}

function NatsIndicator({ url, status }: { url: string; status: NatsStatus }) {
  return (
    <div className="text-right mr-2">
      <div className="flex items-center gap-2 justify-end">
        <p className="text-xs text-gray-400 uppercase tracking-wider font-semibold">Bootstrap Node</p>
        <div 
          className={`w-2.5 h-2.5 rounded-full ${
            status === 'connected' ? 'bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.6)]' : 
            status === 'disconnected' ? 'bg-red-500 shadow-[0_0_6px_rgba(239,68,68,0.6)]' : 
            'bg-yellow-400 animate-pulse'
          }`}
          title={status === 'connected' ? 'Connected to NATS' : 
                 status === 'disconnected' ? 'Cannot connect to NATS' : 
                 'Checking connection...'}
        />
      </div>
      <p className="text-sm font-mono text-gray-700" title={url}>
        {url.length > 30 ? url.substring(0, 30) + '...' : url}
      </p>
    </div>
  );
}
