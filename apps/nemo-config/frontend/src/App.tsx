import { useState, useEffect, useCallback } from 'react';
import axios from 'axios';
import type { Template, ServiceStatus, Mode } from './definitions';
type NatsStatus = 'connected' | 'disconnected' | 'checking';
import { useNatsUrl } from './hooks/useNatsUrl';
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
  const [natsStatus, setNatsStatus] = useState<NatsStatus>('checking');
  const [showSettings, setShowSettings] = useState(false);
  const [natsUrlInput, setNatsUrlInput] = useState(natsUrl);

  // Fetch initial data
  useEffect(() => {
    fetchData();
  }, []);

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
    const [tplRes, hostsRes] = await Promise.all([
      axios.get(`${API_URL}/catalog`),
      axios.get(`${API_URL}/ssh-hosts`)
    ]);

    setTemplates(tplRes.data);
    setHosts(hostsRes.data);
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
      consoleOutput: []
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

  const handleDeploy = async (tabId: string) => {
    const tab = tabs.find(t => t.id === tabId);
    const template = templates.find(t => t.id === tabId);
    if (!tab || !template) return;

    setDeploying(tabId);
    setStatus(prev => ({ ...prev, [tabId]: 'deploying' }));
    appendConsole(tabId, `Starting ${tab.mode} operation...`);

    try {
      if (tab.mode === 'existing') {
        await axios.post(`${API_URL}/register-existing`, {
          service_id: tabId,
          connection_url: tab.existingUrl,
          nats_url: natsUrl,
          metadata: tab.formValues
        });
        appendConsole(tabId, 'Successfully registered existing instance');
      } else {
        await axios.post(`${API_URL}/deploy`, {
          target_host: tab.selectedHost,
          service_id: tabId,
          template,
          env_values: tab.formValues,
          nats_url: natsUrl,
          mode: 'deploy'
        });
        appendConsole(tabId, 'Deployment completed successfully');
      }

      setStatus(prev => ({ ...prev, [tabId]: 'healthy' }));
    } catch (err: any) {
      appendConsole(tabId, `Error: ${err.message || 'Deployment failed'}`);
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
    } catch (err) {
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
              onModeChange={(mode) => updateTab(activeTab.id, { mode })}
              onFormChange={(key, value) => updateTab(activeTab.id, {
                formValues: { ...activeTab.formValues, [key]: value }
              })}
              onHostChange={(host) => updateTab(activeTab.id, { selectedHost: host })}
              onExistingUrlChange={(url) => updateTab(activeTab.id, { existingUrl: url })}
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
          onChange={setNatsUrlInput}
          onSave={() => {
            setNatsUrl(natsUrlInput);
            setShowSettings(false);
          }}
          onCancel={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}
