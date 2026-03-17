import { Play, Activity, Server, Link2 } from 'lucide-react';
import type { Mode, ServiceStatus } from '../definitions';
import type { TestStatus } from './ExistingModeForm';

interface DeployButtonProps {
  mode: Mode;
  status: ServiceStatus;
  isDeploying: boolean;
  selectedHost: string;
  testStatus: TestStatus;
  onDeploy: () => void;
}

export function DeployButton({
  mode,
  status,
  isDeploying,
  selectedHost,
  testStatus,
  onDeploy
}: DeployButtonProps) {
  const isHealthy = status === 'healthy';
  const isExistingMode = mode === 'existing';
  const canRegisterExisting = !isExistingMode || testStatus === 'success';

  return (
    <div className="pt-4">
      <button
        onClick={onDeploy}
        disabled={isDeploying || !canRegisterExisting}
        className={`flex items-center gap-2 px-6 py-2.5 rounded-md shadow-sm text-sm font-medium text-white transition-colors ${isHealthy ? 'bg-green-600 hover:bg-green-700' : 'bg-blue-600 hover:bg-blue-700'
          } disabled:opacity-50 disabled:cursor-not-allowed`}
      >
        {isDeploying ? (
          <><Activity size={16} className="animate-spin" /> {mode === 'deploy' ? 'Deploying...' : 'Registering...'}</>
        ) : isHealthy ? (
          <>{mode === 'deploy' ? <Server size={16} /> : <Link2 size={16} />} Update Config</>
        ) : (
          <>
            {mode === 'deploy' ? <Play size={16} /> : <Link2 size={16} />}
            {mode === 'deploy' ? `Deploy to ${selectedHost || 'localhost'}` : 'Register Existing Instance'}
          </>
        )}
      </button>
      {isExistingMode && !canRegisterExisting && (
        <p className="text-xs text-amber-600 mt-2">
          Please test the connection successfully before registering.
        </p>
      )}
    </div>
  );
}
