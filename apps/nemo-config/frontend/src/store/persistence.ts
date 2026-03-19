import { subscribe } from 'valtio';
import { store } from './index';

const STORAGE_KEYS = {
  consulUrl: 'nemo-consul-url',
  deployPath: 'nemo-deploy-path',
};

export const loadPersistedState = (): void => {
  try {
    const consulUrl = localStorage.getItem(STORAGE_KEYS.consulUrl);
    const deployPath = localStorage.getItem(STORAGE_KEYS.deployPath);
    
    store.consulUrl = consulUrl || 'http://localhost:8500';
    store.deployPath = deployPath || '~/workspace/nemo';
  } catch (err) {
    console.error('Failed to load persisted state:', err);
    store.consulUrl = 'http://localhost:8500';
    store.deployPath = '~/workspace/nemo';
  }
};

let unsubscribe: (() => void) | null = null;

export const startPersistence = (): void => {
  if (unsubscribe) return;
  
  unsubscribe = subscribe(store, () => {
    try {
      localStorage.setItem(STORAGE_KEYS.consulUrl, store.consulUrl);
      localStorage.setItem(STORAGE_KEYS.deployPath, store.deployPath);
    } catch (err) {
      console.error('Failed to persist state:', err);
    }
  });
};

export const stopPersistence = (): void => {
  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }
};
