import { useSnapshot } from 'valtio';
import { store, selectTabContentState, selectActiveTab, selectActiveTemplate } from '../../store';
import { TabError } from './states/Error';
import { TabChecking } from './states/Checking';
import { TabHealthy } from './states/Healthy';
import { TabLoadingDetails } from './states/LoadingDetails';
import { TabDeployForm } from './states/DeployForm';
import { TabExistingForm } from './states/ExistingForm';
import { TabHeader } from './Header';
import { Console } from '../Console';
import type { TabContentStateKey } from '../../store/selectors';

const stateRegistry = {
  error: TabError,
  checking: TabChecking,
  healthy: TabHealthy,
  'loading-details': TabLoadingDetails,
  deploy: TabDeployForm,
  existing: TabExistingForm,
} as const;

export const TabContent = () => {
  const snap = useSnapshot(store);
  const stateKey = selectTabContentState(snap, snap.activeTabId);
  const activeTab = selectActiveTab(snap);
  const template = selectActiveTemplate(snap);

  // Don't render if no active tab
  if (!activeTab || !template) {
    return (
      <div className="flex-1 flex items-center justify-center bg-gray-50">
        <p className="text-gray-400">Select a service from the sidebar to open</p>
      </div>
    );
  }

  const Component = stateRegistry[stateKey as TabContentStateKey];
  const logs = snap.logs[snap.activeTabId || ''] || [];
  const consoleMode = snap.consoleMode[snap.activeTabId || ''] || 'deployment';
  const containerNotFound = activeTab?.instanceDetails?.containerStatus === 'not_found';
  const showOverlay = stateKey === 'error' || stateKey === 'checking';
  const showButton = stateKey === 'deploy' || stateKey === 'existing';

  return (
    <div className="h-full flex flex-col relative">
      <TabHeader />

      {/* Upper: Form or Instance Details */}
      <div className="flex-1 overflow-y-auto p-6 bg-white relative">
        {showOverlay && <Component />}

        <div className="max-w-2xl space-y-5">
          {!showOverlay && <Component />}

          {showButton && <DeployButton />}
        </div>
      </div>

      {/* Lower: Console */}
      <div className="h-[33vh] flex-shrink-0 border-t border-gray-200">
        <Console
          output={[...logs]}
          mode={consoleMode as 'deployment' | 'container'}
          isLoading={snap.isLoadingContainerLogs}
          containerNotFound={containerNotFound}
        />
      </div>
    </div>
  );
};

// DeployButton component inline since it's small
import { Play, Link2, Activity } from 'lucide-react';
import { useMutations } from '../../hooks/useMutations';
import { selectCanRegisterExisting } from '../../store/selectors';

const DeployButton = () => {
  const snap = useSnapshot(store);
  const activeTab = selectActiveTab(snap);
  const template = selectActiveTemplate(snap);
  const isDeploying = snap.deploying === snap.activeTabId;
  const isHealthy = !!snap.configs[`nemo.${snap.activeTabId}.url`];
  const canRegisterExisting = selectCanRegisterExisting(snap, snap.activeTabId);
  const { executeDeploy } = useMutations();

  if (!activeTab || !template) return null;

  const handleDeploy = () => {
    if (snap.activeTabId) {
      executeDeploy.mutate(snap.activeTabId);
    }
  };

  const getButtonContent = () => {
    if (isDeploying) {
      return (
        <>
          <Activity size={16} className="animate-spin" />
          {activeTab.mode === 'existing' ? 'Registering...' : 'Deploying...'}
        </>
      );
    }

    if (isHealthy) {
      return 'Update Config';
    }

    if (activeTab.mode === 'deploy') {
      return (
        <>
          <Play size={16} />
          Deploy to {activeTab.selectedHost}
        </>
      );
    }

    return (
      <>
        <Link2 size={16} />
        Register Existing Instance
      </>
    );
  };

  const isDisabled = executeDeploy.isPending || (activeTab.mode === 'existing' && !canRegisterExisting && !isHealthy);

  return (
    <div className="pt-4">
      <button
        onClick={handleDeploy}
        disabled={isDisabled}
        className={`w-full flex items-center justify-center gap-2 px-4 py-3 rounded-md shadow-sm text-sm font-medium transition-colors ${isHealthy
            ? 'bg-green-600 hover:bg-green-700 text-white'
            : 'bg-blue-600 hover:bg-blue-700 text-white'
          } disabled:opacity-50 disabled:cursor-not-allowed`}
      >
        {getButtonContent()}
      </button>

      {activeTab.mode === 'existing' && !canRegisterExisting && !isHealthy && (
        <p className="text-xs text-amber-600 mt-2 text-center">
          Please test the connection successfully before registering.
        </p>
      )}
    </div>
  );
};
