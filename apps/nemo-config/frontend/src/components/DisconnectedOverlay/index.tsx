import { useSnapshot } from 'valtio';
import { store } from '../../store';
import { openSettings } from '../../store/actions';
import { retryConsulHealth } from '../../hooks/useServerSync';
import { AlertTriangle, Settings, Loader2 } from 'lucide-react';

export const DisconnectedOverlay = () => {
  const snap = useSnapshot(store);

  if (snap.consulStatus !== 'disconnected' || snap.showSettings) return null;

  const handleRetry = () => {
    if (!snap.consulUrl) return;
    store.retryingConnection = true;
    store.consulError = null;
    retryConsulHealth(snap.consulUrl);
  };

  const isRetrying = snap.retryingConnection;

  return (
    <div className="fixed inset-0 bg-black/80 z-[60] flex items-center justify-center">
      <div className="bg-white rounded-xl shadow-2xl p-8 max-w-md mx-4 text-center">
        <div className="flex justify-center mb-4">
          <div className="p-4 bg-red-100 rounded-full">
            <AlertTriangle size={48} className="text-red-500" />
          </div>
        </div>
        
        <h2 className="text-xl font-semibold text-gray-900 mb-2">
          Consul Disconnected
        </h2>
        
        <p className="text-gray-600 mb-4">
          {isRetrying 
            ? 'Attempting to reconnect...'
            : 'Unable to connect to the Consul server. Please verify the Consul URL in your settings or check your network connection.'}
        </p>
        
        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          <button
            onClick={() => openSettings()}
            disabled={isRetrying}
            className="inline-flex items-center justify-center gap-2 px-5 py-2.5 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors"
          >
            <Settings size={18} />
            Update Settings
          </button>
          
          <button
            onClick={handleRetry}
            disabled={isRetrying || !snap.consulUrl}
            className="inline-flex items-center justify-center gap-2 px-5 py-2.5 bg-gray-100 hover:bg-gray-200 disabled:bg-gray-300 disabled:cursor-not-allowed text-gray-700 font-medium rounded-lg transition-colors"
          >
            {isRetrying ? (
              <>
                <Loader2 size={18} className="animate-spin" />
                Retrying...
              </>
            ) : (
              'Retry Connection'
            )}
          </button>
        </div>
        
        {snap.consulUrl && (
          <div className="mt-4 text-sm text-gray-500">
            <p>
              Current URL: <code className="bg-gray-100 px-1.5 py-0.5 rounded">{snap.consulUrl}</code>
            </p>
            {snap.consulError && !isRetrying && (
              <p className="mt-1 text-red-600">
                Error: {snap.consulError}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
