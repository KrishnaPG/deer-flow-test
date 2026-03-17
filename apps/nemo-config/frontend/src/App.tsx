import { useState, useEffect } from 'react';
import axios from 'axios';
import { Database, Box, Activity, MessageSquare, Download, Play, CheckCircle } from 'lucide-react';

const API_URL = 'http://localhost:3001/api';

const ICONS: Record<string, any> = {
  'database': Database,
  'box': Box,
  'activity': Activity,
  'message-square': MessageSquare
};

export default function App() {
  const [templates, setTemplates] = useState<any[]>([]);
  const [hosts, setHosts] = useState<string[]>(['localhost']);
  const [deploying, setDeploying] = useState<string | null>(null);
  const [status, setStatus] = useState<Record<string, 'unconfigured' | 'deploying' | 'healthy'>>({});
  
  // State for form values per service
  const [formValues, setFormValues] = useState<Record<string, Record<string, string>>>({});
  const [selectedHost, setSelectedHost] = useState<Record<string, string>>({});

  useEffect(() => {
    fetchData();
  }, []);

  const fetchData = async () => {
    try {
      const [tplRes, hostsRes] = await Promise.all([
        axios.get(`${API_URL}/catalog`),
        axios.get(`${API_URL}/ssh-hosts`)
      ]);
      setTemplates(tplRes.data);
      setHosts(hostsRes.data);
      
      // Initialize forms with defaults
      const initialForms: any = {};
      const initialHosts: any = {};
      tplRes.data.forEach((tpl: any) => {
        initialForms[tpl.id] = {};
        tpl.env_vars.forEach((env: any) => {
          if (env.default) initialForms[tpl.id][env.key] = env.default;
        });
        // Option D: Use remembered host from NATS metadata if available, otherwise 'localhost'
        initialHosts[tpl.id] = tpl.last_host || 'localhost'; 
        
        // Assume unconfigured to start
        setStatus(prev => ({...prev, [tpl.id]: 'unconfigured'}));
      });
      setFormValues(initialForms);
      setSelectedHost(initialHosts);
    } catch (err) {
      console.error("Failed to fetch data", err);
    }
  };

  const handleDeploy = async (tpl: any) => {
    setDeploying(tpl.id);
    setStatus(prev => ({...prev, [tpl.id]: 'deploying'}));
    try {
      await axios.post(`${API_URL}/deploy`, {
        target_host: selectedHost[tpl.id],
        service_id: tpl.id,
        template: tpl,
        env_values: formValues[tpl.id],
        nats_url: 'nats://localhost:4222' // Assuming bootstrap is local for this demo
      });
      setStatus(prev => ({...prev, [tpl.id]: 'healthy'}));
    } catch (err) {
      console.error("Deployment failed", err);
      setStatus(prev => ({...prev, [tpl.id]: 'unconfigured'}));
    } finally {
      setDeploying(null);
    }
  };

  const handleExport = async () => {
    const res = await axios.get(`${API_URL}/export-env`);
    const blob = new Blob([res.data], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = '.env.export';
    a.click();
  };

  return (
    <div className="min-h-screen bg-gray-50 p-8 font-sans">
      <div className="max-w-5xl mx-auto">
        <div className="flex justify-between items-center mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">🦑 Nemo-Config</h1>
            <p className="text-gray-500">Infrastructure Control Plane</p>
          </div>
          <button 
            onClick={handleExport}
            className="flex items-center gap-2 bg-gray-800 text-white px-4 py-2 rounded-md hover:bg-gray-700"
          >
            <Download size={18} /> Export .env (Eject)
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {templates.map(tpl => {
            const Icon = ICONS[tpl.icon] || Box;
            const currentStatus = status[tpl.id] || 'unconfigured';
            
            return (
              <div key={tpl.id} className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                <div className="flex justify-between items-start mb-4">
                  <div className="flex items-center gap-3">
                    <div className={`p-2 rounded-md ${currentStatus === 'healthy' ? 'bg-green-100 text-green-600' : 'bg-blue-100 text-blue-600'}`}>
                      <Icon size={24} />
                    </div>
                    <div>
                      <h3 className="font-semibold text-lg">{tpl.name}</h3>
                      <p className="text-sm text-gray-500">Port: {tpl.default_port}</p>
                    </div>
                  </div>
                  {currentStatus === 'healthy' && <CheckCircle className="text-green-500" size={24} />}
                </div>

                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Target Host</label>
                    <select 
                      className="w-full border-gray-300 rounded-md shadow-sm p-2 border"
                      value={selectedHost[tpl.id] || 'localhost'}
                      onChange={e => setSelectedHost({...selectedHost, [tpl.id]: e.target.value})}
                      disabled={currentStatus === 'deploying'}
                    >
                      {hosts.map(h => <option key={h} value={h}>{h}</option>)}
                    </select>
                  </div>

                  {tpl.env_vars.map((env: any) => (
                    <div key={env.key}>
                      <label className="block text-sm font-medium text-gray-700 mb-1">{env.description || env.key}</label>
                      <input 
                        type={env.secret ? 'password' : 'text'}
                        className="w-full border-gray-300 rounded-md shadow-sm p-2 border"
                        value={formValues[tpl.id]?.[env.key] || ''}
                        onChange={e => setFormValues({
                          ...formValues, 
                          [tpl.id]: {...formValues[tpl.id], [env.key]: e.target.value}
                        })}
                        disabled={currentStatus === 'deploying'}
                      />
                    </div>
                  ))}

                  <button
                    onClick={() => handleDeploy(tpl)}
                    disabled={currentStatus === 'deploying'}
                    className={`w-full flex justify-center items-center gap-2 px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white ${
                      currentStatus === 'healthy' ? 'bg-green-600 hover:bg-green-700' : 'bg-blue-600 hover:bg-blue-700'
                    } disabled:opacity-50`}
                  >
                    {currentStatus === 'deploying' ? (
                      <span className="animate-pulse">Deploying...</span>
                    ) : currentStatus === 'healthy' ? (
                      <>Redeploy</>
                    ) : (
                      <><Play size={16} /> Deploy Remote</>
                    )}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
