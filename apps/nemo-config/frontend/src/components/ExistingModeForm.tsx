import type { Template } from '../definitions';

interface ExistingModeFormProps {
  template: Template;
  existingUrl: string;
  formValues: Record<string, string>;
  isDeploying: boolean;
  onExistingUrlChange: (url: string) => void;
  onFormChange: (key: string, value: string) => void;
}

export function ExistingModeForm({
  template,
  existingUrl,
  formValues,
  isDeploying,
  onExistingUrlChange,
  onFormChange
}: ExistingModeFormProps) {
  return (
    <>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1.5">Connection URL</label>
        <input
          type="text"
          className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
          value={existingUrl}
          onChange={e => onExistingUrlChange(e.target.value)}
          disabled={isDeploying}
          placeholder={template.connection_url_pattern || 'e.g., postgres://user:pass@host:5432/db'}
        />
        <p className="text-xs text-gray-500 mt-1">Full connection string to existing {template.name} instance</p>
      </div>

      {template.env_vars.some(e => e.secret) && (
        <div className="bg-amber-50 border border-amber-200 rounded-md p-4">
          <h4 className="text-sm font-medium text-amber-900 mb-2">Optional Credentials</h4>
          <div className="grid grid-cols-2 gap-3">
            {template.env_vars.filter(e => e.secret).map((env) => (
              <div key={env.key}>
                <label className="block text-xs font-medium text-gray-700 mb-1">
                  {env.description || env.key}
                </label>
                <input
                  type="password"
                  className="w-full border-gray-300 rounded-md shadow-sm p-2 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-xs"
                  value={formValues[env.key] || ''}
                  onChange={e => onFormChange(env.key, e.target.value)}
                  disabled={isDeploying}
                />
              </div>
            ))}
          </div>
        </div>
      )}
    </>
  );
}
