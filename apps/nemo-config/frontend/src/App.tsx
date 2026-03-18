import { useEffect } from 'react';
import { QueryClientProvider } from '@tanstack/react-query';
import { useServerSync, queryClient } from './hooks/useServerSync';
import { loadPersistedState, startPersistence } from './store/persistence';
import { useWebSocketManager } from './hooks/useWebSocketManager';
import { useLogPollingManager } from './hooks/useLogPollingManager';
import { useInstanceDetailsLoader } from './hooks/useInstanceDetailsLoader';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { TabBar } from './components/TabBar';
import { TabContent } from './components/TabContent';
import { Settings } from './components/Settings';
import './App.css';

function AppContent() {
  // Sync TanStack Query data to Valtio store
  useServerSync();
  
  // Manage WebSocket connection
  useWebSocketManager();
  
  // Manage log polling
  useLogPollingManager();

  // Load instance details when tab becomes active
  useInstanceDetailsLoader();

  // Load persisted settings on mount
  useEffect(() => {
    loadPersistedState();
    startPersistence();
  }, []);
  
  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Header />
      <div className="flex-1 flex overflow-hidden">
        <Sidebar />
        <div className="flex-1 flex flex-col bg-white">
          <TabBar />
          <TabContent />
        </div>
      </div>
      <Settings />
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
    </QueryClientProvider>
  );
}

export default App;
