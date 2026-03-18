import { useSnapshot } from 'valtio';
import { store, selectSidebarState } from '../../store';
import { SidebarLoading } from './states/Loading';
import { SidebarError } from './states/Error';
import { SidebarEmpty } from './states/Empty';
import { SidebarList } from './states/List';
import type { SidebarStateKey } from '../../store/selectors';

const stateRegistry = {
  loading: SidebarLoading,
  error: SidebarError,
  empty: SidebarEmpty,
  list: SidebarList,
} as const;

export const Sidebar = () => {
  const snap = useSnapshot(store);
  const stateKey = selectSidebarState(snap);
  const Component = stateRegistry[stateKey as SidebarStateKey];
  
  return (
    <div className="w-72 bg-gray-50 border-r border-gray-200 flex flex-col h-full">
      <div className="px-4 py-3 border-b border-gray-200 bg-gray-100">
        <h2 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">Services</h2>
      </div>
      <div className="flex-1 overflow-y-auto">
        <Component />
      </div>
    </div>
  );
};
