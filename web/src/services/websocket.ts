import type { WsMessage } from '../types';

type MessageHandler = (message: WsMessage) => void;
type TokenProvider = () => string | null;

export class WebSocketManager {
  private ws: WebSocket | null = null;
  private handlers: Set<MessageHandler> = new Set();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private baseReconnectDelay = 1000;
  private shouldReconnect = true;
  private tokenProvider: TokenProvider;

  constructor(tokenProvider: TokenProvider) {
    this.tokenProvider = tokenProvider;
  }

  private buildUrl(): string | null {
    const token = this.tokenProvider();
    if (!token) return null;
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    return `${protocol}//${host}/ws/dashboard?token=${encodeURIComponent(token)}`;
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    const url = this.buildUrl();
    if (!url) {
      console.warn('[WS] No token available, skipping connect');
      return;
    }

    try {
      this.ws = new WebSocket(url);

      this.ws.onopen = () => {
        console.log('[WS] Connected');
        this.reconnectAttempts = 0;
      };

      this.ws.onmessage = (event) => {
        try {
          const message: WsMessage = JSON.parse(event.data);
          this.handlers.forEach((handler) => handler(message));
        } catch (err) {
          console.error('[WS] Failed to parse message:', err);
        }
      };

      this.ws.onclose = (event) => {
        console.log('[WS] Disconnected:', event.code, event.reason);
        // 4001 = server-side token expired indicator; 1008 = policy violation (auth fail)
        if (event.code === 4001 || event.code === 1008) {
          console.warn('[WS] Auth failure, triggering reconnect with fresh token');
        }
        if (this.shouldReconnect) {
          this.scheduleReconnect();
        }
      };

      this.ws.onerror = (error) => {
        console.error('[WS] Error:', error);
      };
    } catch (err) {
      console.error('[WS] Connection failed:', err);
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.warn('[WS] Max reconnect attempts reached');
      return;
    }

    // If no token available, stop reconnecting
    if (!this.tokenProvider()) {
      console.warn('[WS] No token for reconnect, stopping');
      return;
    }

    const delay = this.baseReconnectDelay * Math.pow(2, this.reconnectAttempts);
    this.reconnectAttempts++;

    console.log(`[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  disconnect(): void {
    this.shouldReconnect = false;

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }

    this.handlers.clear();
  }

  subscribe(handler: MessageHandler): () => void {
    this.handlers.add(handler);
    return () => {
      this.handlers.delete(handler);
    };
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

let instance: WebSocketManager | null = null;

export function getWebSocketManager(tokenProvider: TokenProvider): WebSocketManager {
  if (!instance) {
    instance = new WebSocketManager(tokenProvider);
  }
  return instance;
}

export function destroyWebSocketManager(): void {
  if (instance) {
    instance.disconnect();
    instance = null;
  }
}
