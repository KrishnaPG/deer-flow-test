import { useState, useEffect, useCallback, useRef } from 'react';
import axios from 'axios';
import type { Template, ServiceStatus, Mode, InstanceDetails } from './definitions';
import type { TestStatus } from './components/ExistingModeForm';
type NatsStatus = 'connected' | 'disconnected' | 'checking';
type ConsoleMode = 'deployment' | 'container';
import { useNatsUrl } from './hooks/useNatsUrl';
import { useDeployPath } from './hooks/useDeployPath';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { TabBar } from './components/TabBar';
import { TabContent } from './components/TabContent';
import { Settings } from './components/Settings';

const API_URL = 'http://localhost:3001/api';

interface TabState {
  id: string;
  name: string;
  mode: Mode;
  formValues: Record<string, string>;
  selectedHost: string;
  existingUrl: string;
  consoleOutput: string[];
  consoleMode: ConsoleMode;
  testStatus: TestStatus;
  testMessage?: string;
  instanceDetails: InstanceDetails | null;
  isProcessing: boolean;
}

export default function App() {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [hosts, setHosts] = useState<string[]>(['localhost']);
  const [status, setStatus] = useState<Record<string, ServiceStatus>>({});
  const [deploying, setDeploying] = useState<string | null>(null);

  // Tab state
  const [tabs, setTabs] = useState<TabState[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);

  const { natsUrl, setNatsUrl } = useNatsUrl();
  const { deployPath, setDeployPath } = useDeployPath();
  const [natsStatus, setNatsStatus] = useState<NatsStatus>('checking');
  const [showSettings, setShowSettings] = useState(false);
  const [natsUrlInput, setNatsUrlInput] = useState(natsUrl);
  const [deployPathInput, setDeployPathInput] = useState(deployPath);

  const wsRef = useRef<WebSocket | null>(null);
  const logPollingRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);

  // WebSocket for real-time deployment logs
  useEffect(() => {
    const connectWebSocket = () => {
      if (wsRef.current?.readyState === WebSocket.OPEN) return;
      
      try {
        const ws = new WebSocket('ws://localhost:3001/ws/logs');
        wsRef.current = ws;
        
        ws.onopen = () => {
          console.log('WebSocket connected');
          reconnectAttemptsRef.current = 0;
        };
        
        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.serviceId && data.message) {
              setTabs(prevTabs => prevTabs.map(tab => 
                tab.id === data.serviceId 
                  ? { ...tab, consoleOutput: [...tab.consoleOutput, data.message] }
                  : tab
              ));
            }
          } catch (err) {
            console.error('Failed to parse WebSocket message', err);
          }
        };

        ws.onerror = (error) => {
          console.error('WebSocket error:', error);
        };

        ws.onclose = () => {
          console.log('WebSocket closed');
          wsRef.current = null;
          
          // Attempt to reconnect if not intentionally closed
          if (reconnectAttemptsRef.current < 5) {
            reconnectAttemptsRef.current++;
            const delay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 10000);
            console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttemptsRef.current})`);
            reconnectTimeoutRef.current = setTimeout(connectWebSocket, delay);
          }
        };
      } catch (err) {
        console.error('Failed to create WebSocket:', err);
      }
    };

    connectWebSocket();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, []);

  // Poll for container logs when service is healthy
  const fetchContainerLogs = useCallback(async (serviceId: string) => {
    try {
      const res = await axios.get(`${API_URL}/services/${serviceId}/logs`, {
        params: { nats_url: natsUrl, tail: 100 }
      });
      return res.data.logs || [];
    } catch (err) {
      console.error('Failed to fetch container logs', err);
      return [];
    }
  }, [natsUrl]);

  // Start/stop log polling based on tab visibility and service status
  useEffect(() => {
    if (!activeTabId) {
      if (logPollingRef.current) {
        clearInterval(logPollingRef.current);
        logPollingRef.current = null;
      }
      return;
    }

    const activeTab = tabs.find(t => t.id === activeTabId);
    if (!activeTab) return;

    const isHealthy = status[activeTabId] === 'healthy';
    const isManaged = activeTab.instanceDetails?.metadata?.managedBy === 'nemo';

    if (isHealthy && isManaged) {
      // Start polling for container logs
      const pollLogs = async () => {
        const logs = await fetchContainerLogs(activeTabId);
        setTabs(prevTabs => prevTabs.map(tab =>
          tab.id === activeTabId
            ? { ...tab, consoleOutput: logs, consoleMode: 'container' as ConsoleMode }
            : tab
        ));
      };
      
      pollLogs();
      logPollingRef.current = setInterval(pollLogs, 1000);

      return () => {
        if (logPollingRef.current) {
          clearInterval(logPollingRef.current);
          logPollingRef.current = null;
        }
      };
    } else {
      if (logPollingRef.current) {
        clearInterval(logPollingRef.current);
        logPollingRef.current = null;
      }
    }
  }, [activeTabId, status, tabs, fetchContainerLogs, natsUrl]);

  const fetchData = useCallback(async () => {
    const [tplRes, hostsRes, configsRes] = await Promise.all([
      axios.get(`${API_URL}/catalog`),
      axios.get(`${API_URL}/ssh-hosts`),
      axios.get(`${API_URL}/configs`, { params: { nats_url: natsUrl } }).catch(() => ({ data: {} }))
    ]);

    setTemplates(tplRes.data);
    setHosts(hostsRes.data);

    // Compute status from configs
    const configs = configsRes.data || {};
    const newStatus: Record<string, ServiceStatus> = {};
    
    tplRes.data.forEach((tpl: Template) => {
      if (configs[`${tpl.id}.url`]) {
        newStatus[tpl.id] = 'healthy';
      }
    });
    
    setStatus(newStatus);
  }, [natsUrl]);

  // Fetch initial data
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // Check NATS health
  useEffect(() => {
    const checkHealth = async () => {
      try {
        const res = await axios.get(`${API_URL}/health/nats`, {
          params: { nats_url: natsUrl },
          timeout: 10000
        });
        setNatsStatus(res.data.connected ? 'connected' : 'disconnected');
      } catch {
        setNatsStatus('disconnected');
      }
    };
    checkHealth();
    const interval = setInterval(checkHealth, 10000);
    return () => clearInterval(interval);
  }, [natsUrl]);

  // Fetch instance details when tab becomes active and service is healthy
  useEffect(() => {
    if (!activeTabId) return;
    
    const fetchDetails = async () => {
      if (status[activeTabId] === 'healthy') {
        try {
          const res = await axios.get(`${API_URL}/services/${activeTabId}/details`, {
            params: { nats_url: natsUrl }
          });
          setTabs(prevTabs => prevTabs.map(tab =>
            tab.id === activeTabId
              ? { ...tab, instanceDetails: res.data as InstanceDetails }
              : tab
          ));
        } catch (err) {
          console.error('Failed to fetch instance details', err);
        }
      }
    };
    
fetchDetails();
  }, [activeTabId, status, natsUrl]);

  const openTab = async (templateId: string) => {
    const template = templates.find(t => t.id === templateId);
    if (!template) return;

    // Check if tab already exists
    const existingTab = tabs.find(t => t.id === templateId);
    if (existingTab) {
      setActiveTabId(templateId);
      // If tab exists but has no instance details and service is healthy, fetch them
      if (!existingTab.instanceDetails && status[templateId] === 'healthy') {
        try {
          const res = await axios.get(`${API_URL}/services/${templateId}/details`, {
            params: { nats_url: natsUrl }
          });
          setTabs(prevTabs => prevTabs.map(tab =>
            tab.id === templateId
              ? { ...tab, instanceDetails: res.data as InstanceDetails }
              : tab
          ));
        } catch (err) {
          console.error('Failed to fetch instance details for existing tab', err);
        }
      }
      return;
    }

    // Create new tab with initial values
    const formValues: Record<string, string> = {};
    template.env_vars.forEach(env => {
      if (env.default) formValues[env.key] = env.default;
    });

    let existingUrl = '';
    if (template.connection_url_pattern) {
      existingUrl = template.connection_url_pattern
        .replace('${HOST}', 'localhost')
        .replace(/\$\{([^}]+)\}/g, (_, key) => formValues[key] || '');
    }

    const newTab: TabState = {
      id: templateId,
      name: template.name,
      mode: 'deploy',
      formValues,
      selectedHost: 'localhost',
      existingUrl,
      consoleOutput: [],
      consoleMode: 'deployment',
      testStatus: 'idle',
      testMessage: undefined,
      instanceDetails: null,
      isProcessing: false
    };

    setTabs([...tabs, newTab]);
    setActiveTabId(templateId);
  };

  const closeTab = (tabId: string) => {
    const newTabs = tabs.filter(t => t.id !== tabId);
    setTabs(newTabs);

    if (activeTabId === tabId) {
      setActiveTabId(newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null);
    }
  };

  const updateTab = (tabId: string, updates: Partial<TabState>) => {
    setTabs(tabs.map(t => t.id === tabId ? { ...t, ...updates } : t));
  };

  const appendConsole = useCallback((tabId: string, message: string) => {
    setTabs(prev => prev.map(t =>
      t.id === tabId
        ? { ...t, consoleOutput: [...t.consoleOutput, `[${new Date().toLocaleTimeString()}] ${message}`], consoleMode: 'deployment' }
        : t
    ));
  }, []);

  const handleTestConnection = async (tabId: string) => {
    const tab = tabs.find(t => t.id === tabId);
    const template = templates.find(t => t.id === tabId);
    if (!tab || !template) return;

    if (!tab.existingUrl.trim()) {
      updateTab(tabId, { testStatus: 'error', testMessage: 'Please enter a connection URL' });
      appendConsole(tabId, 'Error: No connection URL provided');
      return;
    }

    updateTab(tabId, { testStatus: 'testing', testMessage: undefined });
    appendConsole(tabId, `Testing connection to ${tab.existingUrl}...`);

    // Get the health_check from the template
    const healthCheck = template.health_check;
    if (!healthCheck) {
      updateTab(tabId, { testStatus: 'error', testMessage: 'No health check configuration for this service' });
      appendConsole(tabId, 'Error: No health check configuration found');
      return;
    }

    try {
      const res = await axios.post(`${API_URL}/test-connection`, {
        service_id: tabId,
        connection_url: tab.existingUrl,
        health_check: healthCheck,
        metadata: tab.formValues
      }, { timeout: 15000 });

      if (res.data.success) {
        updateTab(tabId, { 
          testStatus: 'success', 
          testMessage: res.data.message || 'Connection successful' 
        });
        appendConsole(tabId, `✓ Connection test successful: ${res.data.message}`);
      } else {
        updateTab(tabId, { 
          testStatus: 'error', 
          testMessage: res.data.message || 'Connection failed' 
        });
        appendConsole(tabId, `✗ Connection test failed: ${res.data.message}`);
      }
    } catch (err) {
      if (axios.isAxiosError(err)) {
        const errorMsg = err.response?.data?.error || err.message || 'Connection test failed';
        updateTab(tabId, { testStatus: 'error', testMessage: errorMsg });
        appendConsole(tabId, `✗ Connection test error: ${errorMsg}`);
      } else {
        const errorMsg = err instanceof Error ? err.message : 'Connection test failed';
        updateTab(tabId, { testStatus: 'error', testMessage: errorMsg });
        appendConsole(tabId, `✗ Connection test error: ${errorMsg}`);
      }
    }
  };

  const handleDeploy = async (tabId: string) => {
    const tab = tabs.find(t => t.id === tabId);
    const template = templates.find(t => t.id === tabId);
    if (!tab || !template) return;

    // For existing mode, require successful test first
    if (tab.mode === 'existing' && tab.testStatus !== 'success') {
      appendConsole(tabId, 'Error: Please test the connection successfully before registering');
      return;
    }

    setDeploying(tabId);
    setStatus(prev => ({ ...prev, [tabId]: 'deploying' }));
    appendConsole(tabId, `Starting ${tab.mode} operation...`);

    try {
      if (tab.mode === 'existing') {
        await axios.post(`${API_URL}/register-existing`, {
          service_id: tabId,
          connection_url: tab.existingUrl,
          nats_url: natsUrl,
          template,
          env_values: tab.formValues
        });
        appendConsole(tabId, 'Successfully registered existing instance');
      } else {
        await axios.post(`${API_URL}/deploy`, {
          target_host: tab.selectedHost,
          service_id: tabId,
          template,
          env_values: tab.formValues,
          nats_url: natsUrl,
          mode: 'deploy',
          deploy_path: deployPath
        });
        appendConsole(tabId, 'Deployment completed successfully');
      }

      setStatus(prev => ({ ...prev, [tabId]: 'healthy' }));
      
      // Fetch instance details after successful deployment
      setTimeout(async () => {
        try {
          const res = await axios.get(`${API_URL}/services/${tabId}/details`, {
            params: { nats_url: natsUrl }
          });
          updateTab(tabId, { instanceDetails: res.data as InstanceDetails });
        } catch (err) {
          console.error('Failed to fetch instance details after deploy', err);
        }
      }, 500);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Deployment failed';
      appendConsole(tabId, `Error: ${errorMessage}`);
      setStatus(prev => ({ ...prev, [tabId]: 'unconfigured' }));
    } finally {
      setDeploying(null);
    }
  };

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const handleContainerAction = async (tabId: string, action: 'stop' | 'start' | 'restart' | 'delete' | 'removeConfig') => {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return;

    updateTab(tabId, { isProcessing: true });
    appendConsole(tabId, `${action.charAt(0).toUpperCase() + action.slice(1)} operation in progress...`);

    try {
      const endpoint = action === 'removeConfig' 
        ? `${API_URL}/services/${tabId}/config`
        : action === 'delete'
          ? `${API_URL}/services/${tabId}/container`
          : `${API_URL}/services/${tabId}/${action}`;
      
      const method = action === 'delete' || action === 'removeConfig' ? 'delete' : 'post';
      
      await axios({
        method,
        url: endpoint,
        params: { nats_url: natsUrl, deploy_path: deployPath }
      });

      appendConsole(tabId, `${action.charAt(0).toUpperCase() + action.slice(1)} completed successfully`);

      if (action === 'delete' || action === 'removeConfig') {
        // Clear status and instance details
        setStatus(prev => {
          const newStatus = { ...prev };
          delete newStatus[tabId];
          return newStatus;
        });
        updateTab(tabId, { 
          instanceDetails: null, 
          consoleOutput: [],
          consoleMode: 'deployment'
        });
      } else {
        // Refresh instance details after stop/start/restart
        setTimeout(async () => {
          try {
            const res = await axios.get(`${API_URL}/services/${tabId}/details`, {
              params: { nats_url: natsUrl }
            });
            updateTab(tabId, { instanceDetails: res.data as InstanceDetails });
          } catch (err) {
            console.error('Failed to refresh instance details', err);
          }
        }, 1000);
      }
    } catch (err) {
      const errorMsg = axios.isAxiosError(err) 
        ? err.response?.data?.error || err.message 
        : err instanceof Error ? err.message : 'Operation failed';
      appendConsole(tabId, `Error: ${errorMsg}`);
    } finally {
      updateTab(tabId, { isProcessing: false });
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
    } catch {
      alert('Failed to export config. Is NATS running?');
    }
  };

  const activeTab = tabs.find(t => t.id === activeTabId);
  const activeTemplate = templates.find(t => t.id === activeTabId);

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Header
        natsUrl={natsUrl}
        natsStatus={natsStatus}
        onSettingsClick={() => {
          setNatsUrlInput(natsUrl);
          setDeployPathInput(deployPath);
          setShowSettings(true);
        }}
        onExport={handleExport}
      />

      <div className="flex-1 flex overflow-hidden">
        <Sidebar
          templates={templates}
          selectedId={activeTabId}
          status={status}
          onSelect={openTab}
        />

        <div className="flex-1 flex flex-col bg-white">
          <TabBar
            tabs={tabs.map(t => ({ id: t.id, name: t.name, status: status[t.id] || 'unconfigured' }))}
            activeTabId={activeTabId}
            onTabClick={setActiveTabId}
            onTabClose={closeTab}
          />

          {activeTab && activeTemplate ? (
            <TabContent
              template={activeTemplate}
              status={status[activeTab.id] || 'unconfigured'}
              mode={activeTab.mode}
              formValues={activeTab.formValues}
              selectedHost={activeTab.selectedHost}
              existingUrl={activeTab.existingUrl}
              hosts={hosts}
              consoleOutput={activeTab.consoleOutput}
              consoleMode={activeTab.consoleMode}
              isDeploying={deploying === activeTab.id}
              testStatus={activeTab.testStatus}
              testMessage={activeTab.testMessage}
              instanceDetails={activeTab.instanceDetails}
              onModeChange={(mode) => {
                const resetTest = mode === 'existing' ? {} : { testStatus: 'idle' as TestStatus, testMessage: undefined };
                updateTab(activeTab.id, { mode, ...resetTest });
              }}
              onFormChange={(key, value) => updateTab(activeTab.id, {
                formValues: { ...activeTab.formValues, [key]: value }
              })}
              onHostChange={(host) => updateTab(activeTab.id, { selectedHost: host })}
              onExistingUrlChange={(url) => {
                updateTab(activeTab.id, { existingUrl: url, testStatus: 'idle', testMessage: undefined });
              }}
              onTestConnection={() => handleTestConnection(activeTab.id)}
              onDeploy={() => handleDeploy(activeTab.id)}
              onCopy={handleCopy}
              onStop={() => handleContainerAction(activeTab.id, 'stop')}
              onStart={() => handleContainerAction(activeTab.id, 'start')}
              onRestart={() => handleContainerAction(activeTab.id, 'restart')}
              onDelete={() => handleContainerAction(activeTab.id, 'delete')}
              onRemoveConfig={() => handleContainerAction(activeTab.id, 'removeConfig')}
              isProcessing={activeTab.isProcessing}
            />
          ) : (
            <div className="flex-1 flex items-center justify-center bg-gray-50">
              <p className="text-gray-400">Select a service from the sidebar to open</p>
            </div>
          )}
        </div>
      </div>

      {showSettings && (
        <Settings
          natsUrl={natsUrlInput}
          deployPath={deployPathInput}
          onNatsUrlChange={setNatsUrlInput}
          onDeployPathChange={setDeployPathInput}
          onSave={() => {
            setNatsUrl(natsUrlInput);
            setDeployPath(deployPathInput);
            setShowSettings(false);
          }}
          onCancel={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}