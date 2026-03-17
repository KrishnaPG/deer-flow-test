import { Play, Activity, Server, Link2 } from 'lucide-react';
import type { Mode, ServiceStatus } from '../definitions';

interface DeployButtonProps {
  mode: Mode;
  status: ServiceStatus;
  isDeploying: boolean;
  selectedHost: string;
  onDeploy: () => void;
}

export function DeployButton({
  mode,
  status,
  isDeploying,
  selectedHost,
  onDeploy
}: DeployButtonProps) {
  const isHealthy = status === 'healthy';

  return (
    <div className="pt-4">
      <button
        onClick={onDeploy}
        disabled={isDeploying}
        className={`flex items-center gap-2 px-6 py-2.5 rounded-md shadow-sm text-sm font-medium text-white transition-colors ${isHealthy ? 'bg-green-600 hover:bg-green-700' : 'bg-blue-600 hover:bg-blue-700'
          } disabled:opacity-50`}
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
    </div>
  );
}
