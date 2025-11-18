import { useEffect, useRef, useState, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';

interface WebSocketMessage {
  type: 'commit' | 'lock_acquired' | 'lock_released' | 'branch_created';
  data: Record<string, unknown>;
}

interface UseWebSocketOptions {
  onMessage?: (message: WebSocketMessage) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

export function useWebSocket(
  namespace: string,
  name: string,
  options: UseWebSocketOptions = {}
) {
  const [isConnected, setIsConnected] = useState(false);
  const [reconnectAttempts, setReconnectAttempts] = useState(0);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const queryClient = useQueryClient();

  const maxReconnectAttempts = 5;
  const baseDelay = 1000;

  const connect = useCallback(() => {
    // Build WebSocket URL
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws/repos/${namespace}/${name}`;

    try {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setIsConnected(true);
        setReconnectAttempts(0);
        options.onConnect?.();
      };

      ws.onclose = () => {
        setIsConnected(false);
        wsRef.current = null;
        options.onDisconnect?.();

        // Attempt to reconnect with exponential backoff
        if (reconnectAttempts < maxReconnectAttempts) {
          const delay = baseDelay * Math.pow(2, reconnectAttempts);
          reconnectTimeoutRef.current = setTimeout(() => {
            setReconnectAttempts((prev) => prev + 1);
          }, delay);
        }
      };

      ws.onerror = () => {
        // Error will trigger onclose
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);

          // Handle different message types
          switch (message.type) {
            case 'commit':
              queryClient.invalidateQueries({ queryKey: ['commits', namespace, name] });
              queryClient.invalidateQueries({ queryKey: ['repo', namespace, name] });
              toast.success(`New commit by ${message.data.author || 'someone'}`);
              break;

            case 'lock_acquired':
              queryClient.invalidateQueries({ queryKey: ['lock', namespace, name] });
              toast(`Project locked by ${message.data.user || 'someone'}`, {
                icon: '🔒',
              });
              break;

            case 'lock_released':
              queryClient.invalidateQueries({ queryKey: ['lock', namespace, name] });
              toast(`Project unlocked by ${message.data.user || 'someone'}`, {
                icon: '🔓',
              });
              break;

            case 'branch_created':
              queryClient.invalidateQueries({ queryKey: ['branches', namespace, name] });
              toast.success(`New branch: ${message.data.name || 'unknown'}`);
              break;

            default:
              // Unknown message type
              break;
          }

          options.onMessage?.(message);
        } catch {
          // Invalid JSON, ignore
        }
      };
    } catch {
      // WebSocket creation failed, will retry
      setIsConnected(false);
    }
  }, [namespace, name, reconnectAttempts, queryClient, options]);

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connect]);

  // Reconnect when attempts change
  useEffect(() => {
    if (reconnectAttempts > 0 && reconnectAttempts <= maxReconnectAttempts) {
      connect();
    }
  }, [reconnectAttempts, connect]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  return {
    isConnected,
    reconnectAttempts,
    disconnect,
  };
}
