import { subscribe } from 'valtio';
import { store } from './index';

const STORAGE_KEYS = {
  natsUrl: 'nemo-nats-url',
  deployPath: 'nemo-deploy-path',
};

export const loadPersistedState = (): void => {
  try {
    const natsUrl = localStorage.getItem(STORAGE_KEYS.natsUrl);
    const deployPath = localStorage.getItem(STORAGE_KEYS.deployPath);
    
    store.natsUrl = natsUrl || 'nats://localhost:4222';
    store.deployPath = deployPath || '~/workspace/nemo';
  } catch (err) {
    console.error('Failed to load persisted state:', err);
    store.natsUrl = 'nats://localhost:4222';
    store.deployPath = '~/workspace/nemo';
  }
};

let unsubscribe: (() => void) | null = null;

export const startPersistence = (): void => {
  if (unsubscribe) return;
  
  unsubscribe = subscribe(store, () => {
    try {
      localStorage.setItem(STORAGE_KEYS.natsUrl, store.natsUrl);
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
