import axios from 'axios';
import type { Template, InstanceDetails } from '../definitions';

const API_URL = 'http://localhost:3001/api';

export const fetchTemplates = async (): Promise<Template[]> => {
  const res = await axios.get(`${API_URL}/catalog`);
  return res.data;
};

export const fetchHosts = async (): Promise<string[]> => {
  const res = await axios.get(`${API_URL}/ssh-hosts`);
  return res.data;
};

export const fetchConfigs = async (natsUrl: string): Promise<Record<string, string>> => {
  const res = await axios.get(`${API_URL}/configs`, { 
    params: { nats_url: natsUrl } 
  }).catch(() => ({ data: {} }));
  return res.data;
};

export const fetchInstanceDetails = async (serviceId: string, natsUrl: string): Promise<InstanceDetails> => {
  const res = await axios.get(`${API_URL}/services/${serviceId}/details`, {
    params: { nats_url: natsUrl }
  });
  return res.data;
};

export const fetchContainerLogs = async (serviceId: string, natsUrl: string): Promise<string[]> => {
  const res = await axios.get(`${API_URL}/services/${serviceId}/logs`, {
    params: { nats_url: natsUrl, tail: 100 }
  });
  return res.data.logs || [];
};

export const checkNatsHealth = async (natsUrl: string): Promise<boolean> => {
  try {
    const res = await axios.get(`${API_URL}/health/nats`, {
      params: { nats_url: natsUrl },
      timeout: 10000
    });
    return res.data.connected;
  } catch {
    return false;
  }
};