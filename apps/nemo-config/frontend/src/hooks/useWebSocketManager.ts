import { useEffect, useRef } from 'react';
import * as actions from '../store/actions';

export const useWebSocketManager = (): void => {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectAttemptsRef = useRef(0);

  useEffect(() => {
    const connectWebSocket = () => {
      if (wsRef.current?.readyState === WebSocket.OPEN) return;
      
      try {
        const ws = new WebSocket('ws://localhost:3001/ws/logs');
        wsRef.current = ws;
        
        ws.onopen = () => {
          console.log('WebSocket connected');
          reconnectAttemptsRef.current = 0;
          actions.setWsConnected(true);
        };
        
        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.serviceId && data.message) {
              actions.appendLog(data.serviceId, data.message);
            }
          } catch (err) {
            console.error('Failed to parse WebSocket message', err);
          }
        };

        ws.onerror = (error) => {
          console.error('WebSocket error:', error);
        };

        ws.onclose = () => {
          console.log('WebSocket closed');
          wsRef.current = null;
          actions.setWsConnected(false);
          
          if (reconnectAttemptsRef.current < 5) {
            reconnectAttemptsRef.current++;
            const delay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 10000);
            console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttemptsRef.current})`);
            reconnectTimeoutRef.current = setTimeout(connectWebSocket, delay);
          }
        };
      } catch (err) {
        console.error('Failed to create WebSocket:', err);
      }
    };

    connectWebSocket();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, []);
};
