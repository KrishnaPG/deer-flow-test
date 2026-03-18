import { Loader2 } from 'lucide-react';

export const TabChecking = () => (
  <div className="absolute inset-0 bg-white/90 z-40 flex items-center justify-center">
    <div className="text-center">
      <Loader2 size={32} className="text-blue-600 animate-spin mx-auto mb-4" />
      <p className="text-gray-600 font-medium">Fetching instance status...</p>
      <p className="text-sm text-gray-500 mt-2">Please wait while we check service health</p>
    </div>
  </div>
);
