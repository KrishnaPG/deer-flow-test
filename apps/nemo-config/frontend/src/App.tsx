import { useState, useEffect, useCallback, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import type { Template, ServiceStatus } from './definitions';
import type { TestStatus } from './components/ExistingModeForm';
type NatsStatus = 'connected' | 'disconnected' | 'checking';

import { useNatsUrl } from './hooks/useNatsUrl';
import { useDeployPath } from './hooks/useDeployPath';
import { useTabs } from './hooks/useTabs';
import { useWebSocket } from './hooks/useWebSocket';
import { useLogPolling } from './hooks/useLogPolling';
import { createHandlers } from './hooks/handlers';
import { 
  fetchTemplates, 
  fetchHosts, 
  fetchConfigs, 
  checkNatsHealth,
  fetchInstanceDetails 
} from './api/services';

import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { TabBar } from './components/TabBar';
import { TabContent } from './components/TabContent';
import { Settings } from './components/Settings';

export default function App() {
  const [hosts, setHosts] = useState<string[]>(['localhost']);

  const [deploying, setDeploying] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  
  const { natsUrl, setNatsUrl } = useNatsUrl();
  const { deployPath, setDeployPath } = useDeployPath();
  const [natsUrlInput, setNatsUrlInput] = useState(natsUrl);
  const [deployPathInput, setDeployPathInput] = useState(deployPath);

  const { tabs, activeTabId, activeTab, openTab, closeTab, updateTab, appendConsole, setActiveTabId } = useTabs();

  // Data fetching with TanStack Query
  const { data: templates = [], isLoading: isLoadingTemplates, error: templatesError } = useQuery({
    queryKey: ['templates'],
    queryFn: fetchTemplates,
  });

  const { data: configs = {}, refetch: refetchConfigs, isLoading: isLoadingConfigs, error: configsError } = useQuery({
    queryKey: ['configs', natsUrl],
    queryFn: () => fetchConfigs(natsUrl),
    enabled: !!natsUrl,
  });

  const { data: natsHealthData, isLoading: isCheckingNatsHealth } = useQuery({
    queryKey: ['natsHealth', natsUrl],
    queryFn: () => checkNatsHealth(natsUrl),
    refetchInterval: 10000,
    enabled: !!natsUrl,
  });

  // Compute service status from configs
  const status = useMemo(() => {
    if (!templates.length) return {};
    const newStatus: Record<string, ServiceStatus> = {};
    templates.forEach((tpl: Template) => {
      if (configs[`${tpl.id}.url`]) newStatus[tpl.id] = 'healthy';
    });
    return newStatus;
  }, [configs, templates]);

  // Fetch hosts once
  useEffect(() => {
    fetchHosts().then(setHosts).catch(console.error);
  }, []);

  // WebSocket for deployment logs
  useWebSocket((serviceId, message) => {
    appendConsole(serviceId, message);
  });

  // Container log polling
  useLogPolling(activeTabId, tabs, status, natsUrl, (tabId, logs) => {
    updateTab(tabId, { consoleOutput: logs });
  });

  // Fetch instance details when tab becomes active
  useEffect(() => {
    if (!activeTabId || status[activeTabId] !== 'healthy') return;
    
    const activeTabData = tabs.find(t => t.id === activeTabId);
    if (activeTabData?.instanceDetails) return;
    
    fetchInstanceDetails(activeTabId, natsUrl)
      .then(details => updateTab(activeTabId, { instanceDetails: details }))
      .catch(console.error);
  }, [activeTabId, status, natsUrl, tabs, updateTab]);

  const handleSelectService = useCallback((templateId: string) => {
    const template = templates.find(t => t.id === templateId);
    if (template) openTab(template);
  }, [templates, openTab]);

  const handlers = createHandlers({
    tabs,
    templates,
    natsUrl,
    deployPath,
    updateTab,
    appendConsole,
    setDeploying,
    refetchConfigs
  });

  const natsStatus: NatsStatus = isCheckingNatsHealth ? 'checking' : natsHealthData ? 'connected' : 'disconnected';
  const activeTemplate = templates.find(t => t.id === activeTabId);

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Header
        natsUrl={natsUrl}
        natsStatus={natsStatus}
        onSettingsClick={() => { setNatsUrlInput(natsUrl); setDeployPathInput(deployPath); setShowSettings(true); }}
        onExport={handlers.handleExport}
      />

      <div className="flex-1 flex overflow-hidden">
        <Sidebar
          templates={templates}
          selectedId={activeTabId}
          status={status}
          onSelect={handleSelectService}
          isLoading={isLoadingTemplates}
          error={templatesError ? 'Failed to load services' : undefined}
          isLoadingConfigs={isLoadingConfigs}
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
              isLoadingConfigs={isLoadingConfigs}
              configsError={configsError ? 'Failed to fetch service status' : undefined}
              onConfigsRetry={() => refetchConfigs()}
              onModeChange={(mode) => updateTab(activeTab.id, { mode, ...(mode === 'existing' ? {} : { testStatus: 'idle' as TestStatus, testMessage: undefined }) })}
              onFormChange={(key, value) => updateTab(activeTab.id, { formValues: { ...activeTab.formValues, [key]: value } })}
              onHostChange={(host) => updateTab(activeTab.id, { selectedHost: host })}
              onExistingUrlChange={(url) => updateTab(activeTab.id, { existingUrl: url, testStatus: 'idle', testMessage: undefined })}
              onTestConnection={() => handlers.handleTestConnection(activeTab.id)}
              onDeploy={() => handlers.handleDeploy(activeTab.id)}
              onCopy={(text) => navigator.clipboard.writeText(text)}
              onStop={() => handlers.handleContainerAction(activeTab.id, 'stop')}
              onStart={() => handlers.handleContainerAction(activeTab.id, 'start')}
              onRestart={() => handlers.handleContainerAction(activeTab.id, 'restart')}
              onDelete={() => handlers.handleContainerAction(activeTab.id, 'delete')}
              onRemoveConfig={() => handlers.handleContainerAction(activeTab.id, 'removeConfig')}
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
          onSave={() => { setNatsUrl(natsUrlInput); setDeployPath(deployPathInput); setShowSettings(false); }}
          onCancel={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}