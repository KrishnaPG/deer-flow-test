import { CONFIG } from '../config';

/**
 * Makes an API request to the backend
 */
export async function apiRequest<T>(
  endpoint: string, 
  options: RequestInit = {}
): Promise<{ data: T; status: number }> {
  const url = `${CONFIG.API_URL}${endpoint}`;
  
  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...(options.headers || {}),
    },
    ...options,
  });
  
  const text = await response.text();
  
  if (!text.trim()) {
    return { data: {} as T, status: response.status };
  }
  
  let data: T;
  try {
    data = JSON.parse(text) as T;
  } catch (error) {
    throw new Error(`Failed to parse JSON from ${url}: ${error.message}`);
  }
  
  return { data, status: response.status };
}

/**
 * GET request helper
 */
export async function get<T>(endpoint: string): Promise<{ data: T; status: number }> {
  return apiRequest<T>(endpoint, { method: 'GET' });
}

/**
 * POST request helper
 */
export async function post<T>(
  endpoint: string, 
  body: any
): Promise<{ data: T; status: number }> {
  return apiRequest<T>(endpoint, {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

/**
 * DELETE request helper
 */
export async function del<T>(
  endpoint: string
): Promise<{ data: T; status: number }> {
  return apiRequest<T>(endpoint, { method: 'DELETE' });
}