import { Server, Link2 } from 'lucide-react';
import type { Template, Mode, ServiceStatus, InstanceDetails } from '../definitions';
import { DeployModeForm } from './DeployModeForm';
import { ExistingModeForm, type TestStatus } from './ExistingModeForm';
import { DeployButton } from './DeployButton';
import { Console } from './Console';
import { InstanceDetailsView } from './InstanceDetails';

interface TabContentProps {
  template: Template;
  status: ServiceStatus;
  mode: Mode;
  formValues: Record<string, string>;
  selectedHost: string;
  existingUrl: string;
  hosts: string[];
  consoleOutput: string[];
  consoleMode: 'deployment' | 'container';
  isDeploying: boolean;
  testStatus: TestStatus;
  testMessage?: string;
  instanceDetails: InstanceDetails | null;
  onModeChange: (mode: Mode) => void;
  onFormChange: (key: string, value: string) => void;
  onHostChange: (host: string) => void;
  onExistingUrlChange: (url: string) => void;
  onTestConnection: () => void;
  onDeploy: () => void;
  onCopy: (text: string) => void;
  onStop: () => void;
  onStart: () => void;
  onRestart: () => void;
  onDelete: () => void;
  onRemoveConfig: () => void;
  isProcessing: boolean;
}

export function TabContent({
  template,
  status,
  mode,
  formValues,
  selectedHost,
  existingUrl,
  hosts,
  consoleOutput,
  consoleMode,
  isDeploying,
  testStatus,
  testMessage,
  instanceDetails,
  onModeChange,
  onFormChange,
  onHostChange,
  onExistingUrlChange,
  onTestConnection,
  onDeploy,
  onCopy,
  onStop,
  onStart,
  onRestart,
  onDelete,
  onRemoveConfig,
  isProcessing
}: TabContentProps) {
  const isHealthy = status === 'healthy';

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="px-6 py-3 border-b border-gray-200 flex items-center justify-between bg-white">
        <div>
          <h2 className="text-lg font-semibold text-gray-900">{template.name}</h2>
          <p className="text-xs text-gray-500 font-mono">Port: {template.default_port}</p>
        </div>
        {!isHealthy && <ModeToggle mode={mode} onChange={onModeChange} />}
      </div>

      {/* Upper: Form or Instance Details */}
      <div className="flex-1 overflow-y-auto p-6 bg-white">
        <div className="max-w-2xl space-y-5">
          {isHealthy ? (
            instanceDetails ? (
              <InstanceDetailsView
                instanceDetails={instanceDetails}
                onCopy={onCopy}
                onStop={onStop}
                onStart={onStart}
                onRestart={onRestart}
                onDelete={onDelete}
                onRemoveConfig={onRemoveConfig}
                isProcessing={isProcessing}
              />
            ) : (
              <div className="flex items-center justify-center py-12">
                <div className="text-center">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
                  <p className="text-gray-500">Loading instance details...</p>
                </div>
              </div>
            )
          ) : (
            <>
              {mode === 'deploy' ? (
                <DeployModeForm
                  hosts={hosts}
                  selectedHost={selectedHost}
                  template={template}
                  formValues={formValues}
                  isDeploying={isDeploying}
                  onHostChange={onHostChange}
                  onFormChange={onFormChange}
                />
              ) : (
                <ExistingModeForm
                  template={template}
                  existingUrl={existingUrl}
                  formValues={formValues}
                  isDeploying={isDeploying}
                  testStatus={testStatus}
                  testMessage={testMessage}
                  onExistingUrlChange={onExistingUrlChange}
                  onFormChange={onFormChange}
                  onTestConnection={onTestConnection}
                />
              )}

              <DeployButton
                mode={mode}
                status={status}
                isDeploying={isDeploying}
                selectedHost={selectedHost}
                testStatus={testStatus}
                onDeploy={onDeploy}
              />
            </>
          )}
        </div>
      </div>

      {/* Lower: Console */}
      <div className="h-48 flex-shrink-0 border-t border-gray-200">
        <Console output={consoleOutput} mode={consoleMode} />
      </div>
    </div>
  );
}

function ModeToggle({ mode, onChange }: { mode: Mode; onChange: (m: Mode) => void }) {
  return (
    <div className="bg-gray-100 p-1 rounded-lg inline-flex">
      <button
        onClick={() => onChange('deploy')}
        className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all ${
          mode === 'deploy' ? 'bg-white text-gray-900 shadow-sm' : 'text-gray-600 hover:text-gray-900'
        }`}
      >
        <Server size={14} /> Deploy New
      </button>
      <button
        onClick={() => onChange('existing')}
        className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all ${
          mode === 'existing' ? 'bg-white text-gray-900 shadow-sm' : 'text-gray-600 hover:text-gray-900'
        }`}
      >
        <Link2 size={14} /> Use Existing
      </button>
    </div>
  );
}