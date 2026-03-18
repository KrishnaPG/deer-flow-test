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
  
  let data: T | null = null;
  try {
    // Try to parse JSON, but handle cases where response might be text
    const contentType = response.headers.get('content-type') || '';
    if (contentType.includes('application/json')) {
      data = await response.json();
    } else {
      const text = await response.text();
      // Try to parse as JSON anyway in case content-type is wrong
      try {
        data = JSON.parse(text) as T;
      } catch {
        // If it's not JSON, return as text (cast to T)
        data = text as unknown as T;
      }
    }
  } catch (error) {
    // If parsing fails, return empty object
    data = {} as T;
  }
  
  return { data: data!, status: response.status };
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