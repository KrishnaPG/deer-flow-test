import { Copy, Play, Square, RotateCcw, Trash2, ExternalLink, Container, Server, Clock, CheckCircle, XCircle } from 'lucide-react';
import type { InstanceDetails } from '../definitions';

interface InstanceDetailsViewProps {
  instanceDetails: InstanceDetails;
  onCopy: (text: string) => void;
  onStop: () => void;
  onStart: () => void;
  onRestart: () => void;
  onDelete: () => void;
  onRemoveConfig: () => void;
  isProcessing: boolean;
}

type ContainerStatus = 'running' | 'stopped' | 'not_found';

function StatusBadge({ status }: { status: ContainerStatus }) {
  const config = {
    running: { color: 'bg-green-100 text-green-800', icon: CheckCircle, text: 'Running' },
    stopped: { color: 'bg-yellow-100 text-yellow-800', icon: Square, text: 'Stopped' },
    not_found: { color: 'bg-red-100 text-red-800', icon: XCircle, text: 'Not Found' }
  }[status];
  
  const Icon = config.icon;
  
  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${config.color}`}>
      <Icon size={12} />
      {config.text}
    </span>
  );
}

function CopyableField({ label, value, onCopy }: { label: string; value: string; onCopy: (text: string) => void }) {
  return (
    <div className="flex flex-col gap-1">
      <label className="text-xs font-medium text-gray-500 uppercase tracking-wide">{label}</label>
      <div className="flex items-center gap-2 bg-gray-50 px-3 py-2 rounded-md border border-gray-200">
        <code className="flex-1 text-sm text-gray-800 font-mono break-all">{value}</code>
        <button
          onClick={() => onCopy(value)}
          className="p-1 hover:bg-gray-200 rounded transition-colors"
          title="Copy to clipboard"
        >
          <Copy size={14} className="text-gray-500" />
        </button>
      </div>
    </div>
  );
}

function formatDate(dateStr: string): string {
  if (!dateStr) return 'N/A';
  try {
    return new Date(dateStr).toLocaleString();
  } catch {
    return dateStr;
  }
}

export function InstanceDetailsView({
  instanceDetails,
  onCopy,
  onStop,
  onStart,
  onRestart,
  onDelete,
  onRemoveConfig,
  isProcessing
}: InstanceDetailsViewProps) {
  const { metadata, connectionUrl, containerStatus } = instanceDetails;
  const isManaged = metadata?.managedBy === 'nemo';
  const isRunning = containerStatus === 'running';
  const isStopped = containerStatus === 'stopped';

  return (
    <div className="space-y-6">
      {/* Header with status */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          {isManaged ? (
            <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
              <Container size={12} />
              Managed by Nemo
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
              <ExternalLink size={12} />
              External Instance
            </span>
          )}
          {isManaged && containerStatus && <StatusBadge status={containerStatus} />}
        </div>
      </div>

      {/* Connection URL */}
      {connectionUrl && (
        <CopyableField label="Connection URL" value={connectionUrl} onCopy={onCopy} />
      )}

      {/* Instance details */}
      {metadata && (
        <div className="grid grid-cols-2 gap-4">
          {isManaged && (
            <>
              <div className="flex flex-col gap-1">
                <label className="text-xs font-medium text-gray-500 uppercase tracking-wide">Container Name</label>
                <div className="flex items-center gap-2">
                  <Container size={14} className="text-gray-400" />
                  <code className="text-sm text-gray-800 font-mono">{metadata.containerName}</code>
                </div>
              </div>
              <div className="flex flex-col gap-1">
                <label className="text-xs font-medium text-gray-500 uppercase tracking-wide">Host</label>
                <div className="flex items-center gap-2">
                  <Server size={14} className="text-gray-400" />
                  <code className="text-sm text-gray-800">{metadata.host}</code>
                </div>
              </div>
            </>
          )}
          {!isManaged && (
            <div className="flex flex-col gap-1">
              <label className="text-xs font-medium text-gray-500 uppercase tracking-wide">Host</label>
              <div className="flex items-center gap-2">
                <Server size={14} className="text-gray-400" />
                <code className="text-sm text-gray-800">{metadata.host}</code>
              </div>
            </div>
          )}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-gray-500 uppercase tracking-wide">Deployed At</label>
            <div className="flex items-center gap-2">
              <Clock size={14} className="text-gray-400" />
              <span className="text-sm text-gray-800">{formatDate(metadata.deployedAt)}</span>
            </div>
          </div>
        </div>
      )}

      {/* Action buttons */}
      <div className="pt-4 border-t border-gray-200">
        {isManaged ? (
          <div className="flex flex-wrap gap-2">
            {isRunning && (
              <button
                onClick={onStop}
                disabled={isProcessing}
                className="flex items-center gap-2 px-4 py-2 bg-yellow-500 hover:bg-yellow-600 text-white rounded-md text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                <Square size={14} />
                Stop
              </button>
            )}
            {isStopped && (
              <button
                onClick={onStart}
                disabled={isProcessing}
                className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-md text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                <Play size={14} />
                Start
              </button>
            )}
            {isRunning && (
              <button
                onClick={onRestart}
                disabled={isProcessing}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                <RotateCcw size={14} />
                Restart
              </button>
            )}
            <button
              onClick={onDelete}
              disabled={isProcessing || !isStopped}
              className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors ml-auto"
            >
              <Trash2 size={14} />
              Delete Container
            </button>
          </div>
        ) : (
          <div className="flex justify-end">
            <button
              onClick={onRemoveConfig}
              disabled={isProcessing}
              className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <Trash2 size={14} />
              Remove Config
            </button>
          </div>
        )}
      </div>
    </div>
  );
}