import { useState, useCallback } from 'react';
import type { Template, Mode, InstanceDetails } from '../definitions';
import type { TestStatus } from '../components/ExistingModeForm';

type ConsoleMode = 'deployment' | 'container';

export interface TabState {
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

export function useTabs() {
  const [tabs, setTabs] = useState<TabState[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);

  const createTab = (template: Template): TabState => {
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

    return {
      id: template.id,
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
  };

  const openTab = useCallback((template: Template) => {
    const existingTab = tabs.find(t => t.id === template.id);
    if (existingTab) {
      setActiveTabId(template.id);
      return existingTab;
    }

    const newTab = createTab(template);
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(template.id);
    return newTab;
  }, [tabs]);

  const closeTab = useCallback((tabId: string) => {
    setTabs(prev => {
      const newTabs = prev.filter(t => t.id !== tabId);
      if (activeTabId === tabId) {
        setActiveTabId(newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null);
      }
      return newTabs;
    });
  }, [activeTabId]);

  const updateTab = useCallback((tabId: string, updates: Partial<TabState>) => {
    setTabs(prev => prev.map(t => t.id === tabId ? { ...t, ...updates } : t));
  }, []);

  const appendConsole = useCallback((tabId: string, message: string) => {
    setTabs(prev => prev.map(t =>
      t.id === tabId
        ? { 
            ...t, 
            consoleOutput: [...t.consoleOutput, `[${new Date().toLocaleTimeString()}] ${message}`], 
            consoleMode: 'deployment' 
          }
        : t
    ));
  }, []);

  const activeTab = tabs.find(t => t.id === activeTabId);

  return {
    tabs,
    activeTabId,
    activeTab,
    openTab,
    closeTab,
    updateTab,
    appendConsole,
    setActiveTabId
  };
}