import { useState, useEffect } from 'react';
import axios from 'axios';
import { Database, Box, Activity, MessageSquare, Download, Play, CheckCircle, ChevronDown, ChevronUp, Link2, Server, Settings } from 'lucide-react';

const API_URL = 'http://localhost:3001/api';
const NATS_URL_STORAGE_KEY = 'nemo-config-nats-url';

const ICONS: Record<string, any> = {
  'database': Database,
  'box': Box,
  'activity': Activity,
  'message-square': MessageSquare
};

interface Template {
  id: string;
  name: string;
  icon: string;
  default_port: number;
  connection_url_pattern?: string;
  env_vars: Array<{
    key: string;
    description: string;
    default?: string;
    secret?: boolean;
  }>;
}

type Mode = 'deploy' | 'existing';
type NatsStatus = 'connected' | 'disconnected' | 'checking';

export default function App() {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [hosts, setHosts] = useState<string[]>(['localhost']);
  const [deploying, setDeploying] = useState<string | null>(null);
  const [status, setStatus] = useState<Record<string, 'unconfigured' | 'deploying' | 'healthy'>>({});
  
  // NATS URL State - load from localStorage
  const [natsUrl, setNatsUrl] = useState<string>(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem(NATS_URL_STORAGE_KEY) || 'nats://localhost:4222';
    }
    return 'nats://localhost:4222';
  });
  const [natsStatus, setNatsStatus] = useState<NatsStatus>('checking');
  const [showSettings, setShowSettings] = useState(false);
  const [natsUrlInput, setNatsUrlInput] = useState(natsUrl);
  
  // Form state
  const [formValues, setFormValues] = useState<Record<string, Record<string, string>>>({});
  const [selectedHost, setSelectedHost] = useState<Record<string, string>>({});
  const [existingUrls, setExistingUrls] = useState<Record<string, string>>({});
  const [modes, setModes] = useState<Record<string, Mode>>({});
  
  // Accordion state
  const [expandedId, setExpandedId] = useState<string | null>(null);

  useEffect(() => {
    fetchData();
  }, []);

  // Persist NATS URL to localStorage whenever it changes
  useEffect(() => {
    localStorage.setItem(NATS_URL_STORAGE_KEY, natsUrl);
  }, [natsUrl]);

  // Check NATS connection health
  useEffect(() => {
    const checkNatsHealth = async () => {
      try {
        const res = await axios.get(`${API_URL}/health/nats`, {
          params: { nats_url: natsUrl },
          timeout: 10000
        });
        setNatsStatus(res.data.connected ? 'connected' : 'disconnected');
      } catch (err) {
        setNatsStatus('disconnected');
      }
    };

    // Check immediately
    checkNatsHealth();
    
    // Then check every 10 seconds
    const interval = setInterval(checkNatsHealth, 10000);
    
    return () => clearInterval(interval);
  }, [natsUrl]);

  const handleSaveNatsUrl = () => {
    setNatsUrl(natsUrlInput);
    setShowSettings(false);
  };

  const fetchData = async () => {
    try {
      const [tplRes, hostsRes] = await Promise.all([
        axios.get(`${API_URL}/catalog`),
        axios.get(`${API_URL}/ssh-hosts`)
      ]);
      setTemplates(tplRes.data);
      setHosts(hostsRes.data);
      
      // Initialize forms
      const initialForms: Record<string, Record<string, string>> = {};
      const initialHosts: Record<string, string> = {};
      const initialModes: Record<string, Mode> = {};
      const initialUrls: Record<string, string> = {};
      
      tplRes.data.forEach((tpl: Template) => {
        initialForms[tpl.id] = {};
        tpl.env_vars.forEach((env) => {
          if (env.default) initialForms[tpl.id][env.key] = env.default;
        });
        initialHosts[tpl.id] = 'localhost';
        initialModes[tpl.id] = 'deploy';
        
        // Pre-fill existing URL pattern if available
        if (tpl.connection_url_pattern) {
          initialUrls[tpl.id] = tpl.connection_url_pattern
            .replace('${HOST}', 'localhost')
            .replace(/\$\{([^}]+)\}/g, (match: string, key: string) => {
              const env = tpl.env_vars.find(e => e.key === key);
              return env?.default || match;
            });
        }
        
        setStatus(prev => ({...prev, [tpl.id]: 'unconfigured'}));
      });
      
      setFormValues(initialForms);
      setSelectedHost(initialHosts);
      setModes(initialModes);
      setExistingUrls(initialUrls);
    } catch (err) {
      console.error("Failed to fetch data", err);
    }
  };

  const handleDeploy = async (tpl: Template, e: React.MouseEvent) => {
    e.stopPropagation();
    setDeploying(tpl.id);
    setStatus(prev => ({...prev, [tpl.id]: 'deploying'}));
    
    try {
      if (modes[tpl.id] === 'existing') {
        // Register existing instance
        await axios.post(`${API_URL}/register-existing`, {
          service_id: tpl.id,
          connection_url: existingUrls[tpl.id],
          nats_url: natsUrl,
          metadata: formValues[tpl.id]
        });
      } else {
        // Deploy new Docker instance
        await axios.post(`${API_URL}/deploy`, {
          target_host: selectedHost[tpl.id],
          service_id: tpl.id,
          template: tpl,
          env_values: formValues[tpl.id],
          nats_url: natsUrl,
          mode: 'deploy'
        });
      }
      
      setStatus(prev => ({...prev, [tpl.id]: 'healthy'}));
      setExpandedId(null);
    } catch (err) {
      console.error("Deployment failed", err);
      setStatus(prev => ({...prev, [tpl.id]: 'unconfigured'}));
    } finally {
      setDeploying(null);
    }
  };

  const handleExport = async () => {
    try {
      const res = await axios.get(`${API_URL}/export-env`, {
        params: { nats_url: natsUrl }
      });
      const blob = new Blob([res.data], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = '.env.export';
      a.click();
    } catch (err) {
      console.error("Export failed", err);
      alert("Failed to export config. Is NATS running?");
    }
  };

  // Sort templates alphabetically
  const sortedTemplates = [...templates].sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="min-h-screen bg-gray-50 p-8 font-sans">
      <div className="max-w-4xl mx-auto">
        {/* Header with Settings */}
        <div className="flex justify-between items-start mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">🦑 Nemo-Config</h1>
            <p className="text-gray-500 mt-1">Infrastructure Control Plane</p>
          </div>
          <div className="flex items-center gap-3">
            {/* NATS URL Display */}
            <div className="text-right mr-2">
              <div className="flex items-center gap-2 justify-end">
                <p className="text-xs text-gray-400 uppercase tracking-wider font-semibold">Bootstrap Node</p>
                {/* Connection Status Indicator */}
                <div 
                  className={`w-2.5 h-2.5 rounded-full ${
                    natsStatus === 'connected' ? 'bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.6)]' : 
                    natsStatus === 'disconnected' ? 'bg-red-500 shadow-[0_0_6px_rgba(239,68,68,0.6)]' : 
                    'bg-yellow-400 animate-pulse'
                  }`}
                  title={natsStatus === 'connected' ? 'Connected to NATS' : 
                         natsStatus === 'disconnected' ? 'Cannot connect to NATS' : 
                         'Checking connection...'}
                />
              </div>
              <p className="text-sm font-mono text-gray-700" title={natsUrl}>
                {natsUrl.length > 35 ? natsUrl.substring(0, 35) + '...' : natsUrl}
              </p>
            </div>
            <button 
              onClick={() => {
                setNatsUrlInput(natsUrl);
                setShowSettings(!showSettings);
              }}
              className="flex items-center gap-2 bg-gray-200 text-gray-700 px-3 py-2 rounded-md hover:bg-gray-300 transition-colors"
              title="Configure NATS URL"
            >
              <Settings size={18} />
            </button>
            <button 
              onClick={handleExport}
              className="flex items-center gap-2 bg-gray-800 text-white px-4 py-2 rounded-md hover:bg-gray-700 transition-colors shadow-sm"
            >
              <Download size={18} /> Export .env
            </button>
          </div>
        </div>

        {/* Settings Panel */}
        {showSettings && (
          <div className="mb-6 bg-white rounded-lg shadow-md border border-gray-200 p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Configuration Settings</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1.5">
                  NATS Bootstrap URL
                </label>
                <input 
                  type="text"
                  className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none transition-shadow font-mono text-sm"
                  value={natsUrlInput}
                  onChange={e => setNatsUrlInput(e.target.value)}
                  placeholder="nats://localhost:4222"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Connection string to your NATS JetStream server. This is used as the central configuration store.
                </p>
              </div>
              <div className="flex justify-end gap-3">
                <button
                  onClick={() => setShowSettings(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSaveNatsUrl}
                  className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 transition-colors"
                >
                  Save Settings
                </button>
              </div>
            </div>
          </div>
        )}

        <div className="flex flex-col gap-3">
          {sortedTemplates.map(tpl => {
            const Icon = ICONS[tpl.icon] || Box;
            const currentStatus = status[tpl.id] || 'unconfigured';
            const isExpanded = expandedId === tpl.id;
            const isUnconfigured = currentStatus === 'unconfigured';
            const currentMode = modes[tpl.id] || 'deploy';
            
            return (
              <div 
                key={tpl.id} 
                className={`bg-white rounded-lg shadow-sm border overflow-hidden transition-all duration-200 ${
                  isUnconfigured ? 'border-gray-200 bg-gray-50/50' : 'border-gray-200'
                }`}
              >
                {/* Accordion Header */}
                <div 
                  className={`flex justify-between items-center p-4 cursor-pointer transition-colors ${
                    isUnconfigured ? 'hover:bg-gray-100/80 opacity-70' : 'hover:bg-gray-50'
                  }`}
                  onClick={() => setExpandedId(isExpanded ? null : tpl.id)}
                >
                  <div className="flex items-center gap-4">
                    <div className={`p-2 rounded-md ${
                      currentStatus === 'healthy' ? 'bg-green-100 text-green-600' : 
                      currentStatus === 'deploying' ? 'bg-blue-100 text-blue-600' : 
                      'bg-gray-200 text-gray-500'
                    }`}>
                      <Icon size={20} />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <h3 className={`font-semibold text-lg ${isUnconfigured ? 'text-gray-600' : 'text-gray-900'}`}>
                          {tpl.name}
                        </h3>
                        {!isUnconfigured && (
                          <span className="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600 font-medium">
                            {currentMode === 'deploy' ? 'Docker' : 'External'}
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-gray-500 font-mono text-xs mt-0.5">Port: {tpl.default_port}</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-6">
                    {/* Status Display */}
                    <div className="flex items-center w-32 justify-end">
                      {currentStatus === 'healthy' && (
                        <span className="flex items-center gap-1.5 text-sm font-medium text-green-600">
                          <CheckCircle size={16} /> Healthy
                        </span>
                      )}
                      {currentStatus === 'deploying' && (
                        <span className="flex items-center gap-1.5 text-sm font-medium text-blue-600 animate-pulse">
                          <Activity size={16} /> {currentMode === 'deploy' ? 'Deploying...' : 'Registering...'}
                        </span>
                      )}
                      {currentStatus === 'unconfigured' && (
                        <span className="text-sm font-medium text-gray-400">
                          Not Configured
                        </span>
                      )}
                    </div>
                    
                    {/* Expand/Collapse Icon */}
                    <div className="text-gray-400 p-1">
                      {isExpanded ? <ChevronUp size={20} /> : <ChevronDown size={20} />}
                    </div>
                  </div>
                </div>

                {/* Accordion Body (Form) */}
                {isExpanded && (
                  <div className="p-6 border-t border-gray-100 bg-white">
                    {/* Mode Toggle */}
                    <div className="mb-6 bg-gray-100 p-1 rounded-lg inline-flex">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setModes({...modes, [tpl.id]: 'deploy'});
                        }}
                        className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                          currentMode === 'deploy' 
                            ? 'bg-white text-gray-900 shadow-sm' 
                            : 'text-gray-600 hover:text-gray-900'
                        }`}
                      >
                        <Server size={16} /> Deploy New
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setModes({...modes, [tpl.id]: 'existing'});
                        }}
                        className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                          currentMode === 'existing' 
                            ? 'bg-white text-gray-900 shadow-sm' 
                            : 'text-gray-600 hover:text-gray-900'
                        }`}
                      >
                        <Link2 size={16} /> Use Existing
                      </button>
                    </div>

                    <div className="space-y-5 max-w-3xl">
                      {currentMode === 'deploy' ? (
                        // Deploy New Mode
                        <>
                          <div>
                            <label className="block text-sm font-medium text-gray-700 mb-1.5">Target Host</label>
                            <select 
                              className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none transition-shadow"
                              value={selectedHost[tpl.id] || 'localhost'}
                              onChange={e => setSelectedHost({...selectedHost, [tpl.id]: e.target.value})}
                              disabled={currentStatus === 'deploying'}
                            >
                              {hosts.map(h => <option key={h} value={h}>{h}</option>)}
                            </select>
                            <p className="text-xs text-gray-500 mt-1">SSH host to deploy Docker container (from ~/.ssh/config)</p>
                          </div>

                          {tpl.env_vars.map((env) => (
                            <div key={env.key}>
                              <label className="block text-sm font-medium text-gray-700 mb-1.5">
                                {env.description || env.key}
                              </label>
                              <input 
                                type={env.secret ? 'password' : 'text'}
                                className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none transition-shadow font-mono text-sm"
                                value={formValues[tpl.id]?.[env.key] || ''}
                                onChange={e => setFormValues({
                                  ...formValues, 
                                  [tpl.id]: {...formValues[tpl.id], [env.key]: e.target.value}
                                })}
                                disabled={currentStatus === 'deploying'}
                                placeholder={env.default ? `Default: ${env.default}` : `Enter ${env.key}`}
                              />
                            </div>
                          ))}
                        </>
                      ) : (
                        // Use Existing Mode
                        <>
                          <div>
                            <label className="block text-sm font-medium text-gray-700 mb-1.5">
                              Connection URL
                            </label>
                            <input 
                              type="text"
                              className="w-full border-gray-300 rounded-md shadow-sm p-2.5 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none transition-shadow font-mono text-sm"
                              value={existingUrls[tpl.id] || ''}
                              onChange={e => setExistingUrls({...existingUrls, [tpl.id]: e.target.value})}
                              disabled={currentStatus === 'deploying'}
                              placeholder={tpl.connection_url_pattern || 'e.g., postgres://user:pass@host:5432/db'}
                            />
                            <p className="text-xs text-gray-500 mt-1">
                              Full connection string to existing {tpl.name} instance
                            </p>
                          </div>

                          <div className="bg-amber-50 border border-amber-200 rounded-md p-4">
                            <h4 className="text-sm font-medium text-amber-900 mb-2">Optional Credentials</h4>
                            <p className="text-xs text-gray-600 mb-3">
                              If your connection URL doesn't include credentials, enter them here:
                            </p>
                            <div className="grid grid-cols-2 gap-3">
                              {tpl.env_vars.filter(e => e.secret).map((env) => (
                                <div key={env.key}>
                                  <label className="block text-xs font-medium text-gray-700 mb-1">
                                    {env.description || env.key}
                                  </label>
                                  <input 
                                    type="password"
                                    className="w-full border-gray-300 rounded-md shadow-sm p-2 border bg-white focus:ring-blue-500 focus:border-blue-500 outline-none font-mono text-xs"
                                    value={formValues[tpl.id]?.[env.key] || ''}
                                    onChange={e => setFormValues({
                                      ...formValues, 
                                      [tpl.id]: {...formValues[tpl.id], [env.key]: e.target.value}
                                    })}
                                    disabled={currentStatus === 'deploying'}
                                  />
                                </div>
                              ))}
                            </div>
                          </div>
                        </>
                      )}

                      <div className="pt-4 flex justify-end">
                        <button
                          onClick={(e) => handleDeploy(tpl, e)}
                          disabled={currentStatus === 'deploying'}
                          className={`flex items-center gap-2 px-6 py-2.5 rounded-md shadow-sm text-sm font-medium text-white transition-colors ${
                            currentStatus === 'healthy' 
                              ? 'bg-green-600 hover:bg-green-700 focus:ring-green-500' 
                              : 'bg-blue-600 hover:bg-blue-700 focus:ring-blue-500'
                          } disabled:opacity-50 focus:outline-none focus:ring-2 focus:ring-offset-2`}
                        >
                          {currentStatus === 'deploying' ? (
                            <><Activity size={16} className="animate-spin" /> 
                              {currentMode === 'deploy' ? 'Deploying...' : 'Registering...'}
                            </>
                          ) : currentStatus === 'healthy' ? (
                            <>{currentMode === 'deploy' ? <Server size={16} /> : <Link2 size={16} />} Update Config</>
                          ) : (
                            <>
                              {currentMode === 'deploy' ? <Play size={16} /> : <Link2 size={16} />}
                              {currentMode === 'deploy' 
                                ? `Deploy to ${selectedHost[tpl.id] || 'localhost'}` 
                                : 'Register Existing Instance'}
                            </>
                          )}
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
