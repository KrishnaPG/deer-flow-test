import { Template } from '../types';

interface DeployModeFormProps {
  hosts: string[];
  selectedHost: string;
  template: Template;
  formValues: Record<string, string>;
  isDeploying: boolean;
  onHostChange: (host: string) => void;
  onFormChange: (key: string, value: string) => void;
}

export function DeployModeForm({
  hosts,
  selectedHost,
  template,
  formValues,
  isDeploying,
  onHostChange,
  onFormChange
}: DeployModeFormProps) {
  return (
    <>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1.5">Target Host</label>
        <select
          className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none"
          value={selectedHost}
          onChange={e => onHostChange(e.target.value)}
          disabled={isDeploying}
        >
          {hosts.map(h => <option key={h} value={h}>{h}</option>)}
        </select>
        <p className="text-xs text-gray-500 mt-1">SSH host to deploy Docker container</p>
      </div>

      {template.env_vars.map((env) => (
        <div key={env.key}>
          <label className="block text-sm font-medium text-gray-700 mb-1.5">
            {env.description || env.key}
          </label>
          <input
            type={env.secret ? 'password' : 'text'}
            className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
            value={formValues[env.key] || ''}
            onChange={e => onFormChange(env.key, e.target.value)}
            disabled={isDeploying}
            placeholder={env.default ? `Default: ${env.default}` : `Enter ${env.key}`}
          />
        </div>
      ))}
    </>
  );
}
