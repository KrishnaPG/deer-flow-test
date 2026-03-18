import { Loader2 } from 'lucide-react';

export const SidebarLoading = () => (
  <div className="flex flex-col items-center justify-center py-12 px-4">
    <Loader2 size={24} className="text-gray-400 animate-spin mb-3" />
    <p className="text-sm text-gray-500">Loading services...</p>
  </div>
);
