import { useState, useEffect } from 'react';

const NATS_URL_STORAGE_KEY = 'nemo-config-nats-url';

export function useNatsUrl() {
  const [natsUrl, setNatsUrl] = useState<string>(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem(NATS_URL_STORAGE_KEY) || 'nats://localhost:4222';
    }
    return 'nats://localhost:4222';
  });

  useEffect(() => {
    localStorage.setItem(NATS_URL_STORAGE_KEY, natsUrl);
  }, [natsUrl]);

  return { natsUrl, setNatsUrl };
}
