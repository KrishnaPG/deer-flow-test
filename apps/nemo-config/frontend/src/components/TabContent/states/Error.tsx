import { AlertCircle, RefreshCw } from 'lucide-react';
import { useSnapshot } from 'valtio';
import { store } from '../../../store';
import { queryClient } from '../../../hooks/useServerSync';

export const TabError = () => {
  const { configsError } = useSnapshot(store);
  
  const handleRetry = () => {
    queryClient.invalidateQueries({ queryKey: ['configs'] });
  };
  
  return (
    <div className="absolute inset-0 bg-white/95 z-50 flex items-center justify-center">
      <div className="text-center px-6">
        <AlertCircle size={48} className="text-red-500 mx-auto mb-4" />
        <h3 className="text-lg font-semibold text-gray-900 mb-2">Service Status Unavailable</h3>
        <p className="text-sm text-gray-600 mb-6 max-w-md">{configsError}</p>
        <p className="text-xs text-gray-500 mb-4">Cannot access service management while status is unknown</p>
        <button
          onClick={handleRetry}
          className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors text-sm font-medium"
        >
          <RefreshCw size={16} />
          Retry
        </button>
      </div>
    </div>
  );
};
