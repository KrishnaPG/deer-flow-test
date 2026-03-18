import { useState, useEffect, useCallback, useRef } from 'react';
import axios from 'axios';
import type { Template, ServiceStatus, Mode } from './definitions';
import type { TestStatus } from './components/ExistingModeForm';
type NatsStatus = 'connected' | 'disconnected' | 'checking';
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
  testStatus: TestStatus;
  testMessage?: string;
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

  // WebSocket for real-time logs
  useEffect(() => {
    if (wsRef.current) return; // Prevent duplicate connections in StrictMode

    const ws = new WebSocket('ws://localhost:3001/ws/logs');
    wsRef.current = ws;
    
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.serviceId && data.message) {
          setTabs(prevTabs => prevTabs.map(tab => 
            tab.id === data.serviceId 
              ? { ...tab, consoleOutput: [...tab.consoleOutput, `[${new Date().toLocaleTimeString()}] ${data.message}`] }
              : tab
          ));
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message', err);
      }
    };

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, []);

  // Fetch initial data
  useEffect(() => {
    fetchData();
  }, [natsUrl]);

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

  const fetchData = async () => {
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
      // If there's a `.url` key in NATS for this service, we consider it healthy/configured
      if (configs[`${tpl.id}.url`]) {
        newStatus[tpl.id] = 'healthy';
      }
    });
    
    setStatus(newStatus);
  };

  const openTab = (templateId: string) => {
    const template = templates.find(t => t.id === templateId);
    if (!template) return;

    // Check if tab already exists
    const existingTab = tabs.find(t => t.id === templateId);
    if (existingTab) {
      setActiveTabId(templateId);
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
      testStatus: 'idle',
      testMessage: undefined
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
        ? { ...t, consoleOutput: [...t.consoleOutput, `[${new Date().toLocaleTimeString()}] ${message}`] }
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
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Deployment failed';
      appendConsole(tabId, `Error: ${errorMessage}`);
      setStatus(prev => ({ ...prev, [tabId]: 'unconfigured' }));
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
              isDeploying={deploying === activeTab.id}
              testStatus={activeTab.testStatus}
              testMessage={activeTab.testMessage}
              onModeChange={(mode) => {
                // Reset test status when switching modes
                const resetTest = mode === 'existing' ? {} : { testStatus: 'idle' as TestStatus, testMessage: undefined };
                updateTab(activeTab.id, { mode, ...resetTest });
              }}
              onFormChange={(key, value) => updateTab(activeTab.id, {
                formValues: { ...activeTab.formValues, [key]: value }
              })}
              onHostChange={(host) => updateTab(activeTab.id, { selectedHost: host })}
              onExistingUrlChange={(url) => {
                // Reset test status when URL changes
                updateTab(activeTab.id, { existingUrl: url, testStatus: 'idle', testMessage: undefined });
              }}
              onTestConnection={() => handleTestConnection(activeTab.id)}
              onDeploy={() => handleDeploy(activeTab.id)}
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
