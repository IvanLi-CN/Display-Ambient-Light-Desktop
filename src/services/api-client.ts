/**
 * HTTP API客户端 - 替代Tauri invoke调用
 * 提供统一的API接口，支持HTTP和WebSocket通信
 */

// API响应类型
export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message?: string;
}

// API错误类型
export interface ApiError {
  code: string;
  message: string;
}

// 配置类型
export interface ApiClientConfig {
  baseUrl: string;
  timeout: number;
  enableWebSocket: boolean;
  webSocketUrl?: string;
}

// WebSocket消息类型
export interface WebSocketMessage {
  type: string;
  data?: any;
  [key: string]: any;
}

// WebSocket事件监听器类型
export type WebSocketEventListener = (message: WebSocketMessage) => void;

/**
 * HTTP API客户端类
 */
export class ApiClient {
  private static instance: ApiClient;
  private config: ApiClientConfig;
  private websocket: WebSocket | null = null;
  private wsEventListeners: Map<string, Set<WebSocketEventListener>> = new Map();
  private wsReconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  private constructor(config: ApiClientConfig) {
    this.config = config;
    
    if (config.enableWebSocket) {
      this.initWebSocket();
    }
  }

  /**
   * 获取API客户端实例
   */
  public static getInstance(config?: ApiClientConfig): ApiClient {
    if (!ApiClient.instance) {
      const defaultConfig: ApiClientConfig = {
        baseUrl: 'http://127.0.0.1:3030',
        timeout: 10000,
        enableWebSocket: true,
        webSocketUrl: 'ws://127.0.0.1:3030/ws'
      };
      ApiClient.instance = new ApiClient(config || defaultConfig);
    }
    return ApiClient.instance;
  }

  /**
   * 初始化WebSocket连接
   */
  private initWebSocket(): void {
    if (!this.config.webSocketUrl) return;

    try {
      this.websocket = new WebSocket(this.config.webSocketUrl);
      
      this.websocket.onopen = () => {
        console.log('🔌 WebSocket连接已建立');
        this.wsReconnectAttempts = 0;
      };

      this.websocket.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          this.handleWebSocketMessage(message);
        } catch (error) {
          console.error('解析WebSocket消息失败:', error);
        }
      };

      this.websocket.onclose = () => {
        console.log('🔌 WebSocket连接已关闭');
        this.websocket = null;
        this.scheduleReconnect();
      };

      this.websocket.onerror = (error) => {
        console.error('WebSocket错误:', error);
      };
    } catch (error) {
      console.error('初始化WebSocket失败:', error);
      this.scheduleReconnect();
    }
  }

  /**
   * 处理WebSocket消息
   */
  private handleWebSocketMessage(message: WebSocketMessage): void {
    const listeners = this.wsEventListeners.get(message.type);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(message);
        } catch (error) {
          console.error('WebSocket事件监听器执行失败:', error);
        }
      });
    }

    // 处理全局监听器
    const globalListeners = this.wsEventListeners.get('*');
    if (globalListeners) {
      globalListeners.forEach(listener => {
        try {
          listener(message);
        } catch (error) {
          console.error('WebSocket全局监听器执行失败:', error);
        }
      });
    }
  }

  /**
   * 安排WebSocket重连
   */
  private scheduleReconnect(): void {
    if (this.wsReconnectAttempts >= this.maxReconnectAttempts) {
      console.error('WebSocket重连次数已达上限，停止重连');
      return;
    }

    this.wsReconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.wsReconnectAttempts - 1);
    
    console.log(`🔄 ${delay}ms后尝试WebSocket重连 (第${this.wsReconnectAttempts}次)`);
    
    setTimeout(() => {
      if (!this.websocket || this.websocket.readyState === WebSocket.CLOSED) {
        this.initWebSocket();
      }
    }, delay);
  }

  /**
   * 添加WebSocket事件监听器
   */
  public onWebSocketEvent(eventType: string, listener: WebSocketEventListener): () => void {
    if (!this.wsEventListeners.has(eventType)) {
      this.wsEventListeners.set(eventType, new Set());
    }
    
    this.wsEventListeners.get(eventType)!.add(listener);
    
    // 返回取消监听的函数
    return () => {
      const listeners = this.wsEventListeners.get(eventType);
      if (listeners) {
        listeners.delete(listener);
        if (listeners.size === 0) {
          this.wsEventListeners.delete(eventType);
        }
      }
    };
  }

  /**
   * 发送WebSocket消息
   */
  public sendWebSocketMessage(message: WebSocketMessage): void {
    if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
      this.websocket.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket未连接，无法发送消息');
    }
  }

  /**
   * 通用HTTP请求方法
   */
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.config.baseUrl}${endpoint}`;
    
    const defaultOptions: RequestInit = {
      headers: {
        'Content-Type': 'application/json',
      },
      ...options,
    };

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

      const response = await fetch(url, {
        ...defaultOptions,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result: ApiResponse<T> = await response.json();
      
      if (!result.success) {
        throw new Error(result.message || 'API调用失败');
      }

      return result.data;
    } catch (error) {
      console.error(`API请求失败 [${endpoint}]:`, error);
      throw error;
    }
  }

  /**
   * GET请求
   */
  public async get<T>(endpoint: string, params?: Record<string, any>): Promise<T> {
    let url = endpoint;
    if (params) {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, String(value));
        }
      });
      if (searchParams.toString()) {
        url += `?${searchParams.toString()}`;
      }
    }

    return this.request<T>(url, { method: 'GET' });
  }

  /**
   * POST请求
   */
  public async post<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  /**
   * PUT请求
   */
  public async put<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  /**
   * DELETE请求
   */
  public async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }

  /**
   * 关闭连接
   */
  public close(): void {
    if (this.websocket) {
      this.websocket.close();
      this.websocket = null;
    }
    this.wsEventListeners.clear();
  }

  /**
   * 获取WebSocket连接状态
   */
  public getWebSocketState(): number | null {
    return this.websocket?.readyState || null;
  }

  /**
   * 检查WebSocket是否已连接
   */
  public isWebSocketConnected(): boolean {
    return this.websocket?.readyState === WebSocket.OPEN;
  }
}

// 导出默认实例
export const apiClient = ApiClient.getInstance();

// 导出便捷方法
export const api = {
  get: <T>(endpoint: string, params?: Record<string, any>) => apiClient.get<T>(endpoint, params),
  post: <T>(endpoint: string, data?: any) => apiClient.post<T>(endpoint, data),
  put: <T>(endpoint: string, data?: any) => apiClient.put<T>(endpoint, data),
  delete: <T>(endpoint: string) => apiClient.delete<T>(endpoint),
  onEvent: (eventType: string, listener: WebSocketEventListener) => apiClient.onWebSocketEvent(eventType, listener),
  sendMessage: (message: WebSocketMessage) => apiClient.sendWebSocketMessage(message),
  isConnected: () => apiClient.isWebSocketConnected(),
};
