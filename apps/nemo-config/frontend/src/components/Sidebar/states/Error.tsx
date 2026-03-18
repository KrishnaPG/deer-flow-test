import { AlertCircle } from 'lucide-react';
import { useSnapshot } from 'valtio';
import { store } from '../../../store';

export const SidebarError = () => {
  const { templatesError } = useSnapshot(store);
  
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4">
      <AlertCircle size={24} className="text-red-500 mb-3" />
      <p className="text-sm text-red-600 text-center">{templatesError}</p>
    </div>
  );
};
