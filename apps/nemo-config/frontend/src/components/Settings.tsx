interface SettingsProps {
  natsUrl: string;
  onChange: (url: string) => void;
  onSave: () => void;
  onCancel: () => void;
}

export function Settings({ natsUrl, onChange, onSave, onCancel }: SettingsProps) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl border border-gray-200 p-6 w-full max-w-lg">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Configuration Settings</h3>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1.5">
              NATS Bootstrap URL
            </label>
            <input
              type="text"
              className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-sm"
              value={natsUrl}
              onChange={e => onChange(e.target.value)}
              placeholder="nats://localhost:4222"
            />
            <p className="text-xs text-gray-500 mt-1">
              Connection string to your NATS JetStream server
            </p>
          </div>
          <div className="flex justify-end gap-3 pt-2">
            <button
              onClick={onCancel}
              className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
            >
              Cancel
            </button>
            <button
              onClick={onSave}
              className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 transition-colors"
            >
              Save Settings
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
