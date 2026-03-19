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

export const fetchConfigs = async (consulUrl: string): Promise<Record<string, string>> => {
  const res = await axios.get(`${API_URL}/configs`, { 
    params: { consul_url: consulUrl } 
  }).catch(() => ({ data: {} }));
  return res.data;
};

export const fetchInstanceDetails = async (serviceId: string, consulUrl: string): Promise<InstanceDetails> => {
  const res = await axios.get(`${API_URL}/services/${serviceId}/details`, {
    params: { consul_url: consulUrl }
  });
  return res.data;
};

export const fetchContainerLogs = async (serviceId: string, consulUrl: string): Promise<string[]> => {
  const res = await axios.get(`${API_URL}/services/${serviceId}/logs`, {
    params: { consul_url: consulUrl, tail: 100 }
  });
  return res.data.logs || [];
};

export const checkConsulHealth = async (consulUrl: string): Promise<{ connected: boolean; error?: string }> => {
  try {
    const res = await axios.get(`${API_URL}/health/consul`, {
      params: { consul_url: consulUrl },
      timeout: 10000
    });
    if (res.data.connected) {
      return { connected: true };
    }
    return { connected: false, error: res.data.error || 'Connection failed' };
  } catch (err) {
    const error = err as any;
    const message = error.response?.status === 0 || !error.response
      ? 'Connection refused or network error'
      : error.response?.data?.message || error.message || 'Unknown error';
    return { connected: false, error: message };
  }
};