/**
 * WebSocket事件监听器组件
 * 处理来自后端的实时更新事件
 */

import { onMount, onCleanup, createSignal } from 'solid-js';
import { adaptiveApi } from '../services/api-adapter';

// WebSocket连接状态
export interface WebSocketStatus {
  connected: boolean;
  lastMessage?: string;
  messageCount: number;
}

// WebSocket事件处理器类型
export interface WebSocketEventHandlers {
  onLedColorsChanged?: (data: any) => void;
  onLedSortedColorsChanged?: (data: any) => void;
  onLedStripColorsChanged?: (data: any) => void;
  onConfigChanged?: (data: any) => void;
  onAmbientLightStateChanged?: (data: any) => void;
  onBoardsChanged?: (data: any) => void;
  onDisplaysChanged?: (data: any) => void;
  onNavigate?: (data: any) => void;
  onConnectionStatusChanged?: (connected: boolean) => void;
  // 支持任意事件名称
  [key: string]: ((data: any) => void) | undefined;
}

interface WebSocketListenerProps {
  handlers?: WebSocketEventHandlers;
  autoConnect?: boolean;
  showStatus?: boolean;
}

/**
 * WebSocket监听器组件
 */
export const WebSocketListener = (props: WebSocketListenerProps) => {
  const [status, setStatus] = createSignal<WebSocketStatus>({
    connected: false,
    messageCount: 0
  });

  const unlistenFunctions: (() => void)[] = [];

  // 更新连接状态
  const updateConnectionStatus = (connected: boolean) => {
    setStatus(prev => ({ ...prev, connected }));
    props.handlers?.onConnectionStatusChanged?.(connected);
  };

  // 注册事件监听器
  const registerEventListeners = async () => {
    try {
      // 初始化API适配器
      await adaptiveApi.initialize();

      // LED颜色变化事件
      if (props.handlers?.onLedColorsChanged) {
        const unlisten = await adaptiveApi.onEvent('LedColorsChanged', (data) => {
          // 移除频繁的颜色变化日志
          setStatus(prev => ({
            ...prev,
            lastMessage: 'LED颜色更新',
            messageCount: prev.messageCount + 1
          }));
          props.handlers?.onLedColorsChanged?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // LED排序颜色变化事件
      if (props.handlers?.onLedSortedColorsChanged) {
        const unlisten = await adaptiveApi.onEvent('LedSortedColorsChanged', (data) => {
          console.log('🌈 LED排序颜色变化:', data);
          setStatus(prev => ({
            ...prev,
            lastMessage: 'LED排序颜色更新',
            messageCount: prev.messageCount + 1
          }));
          props.handlers?.onLedSortedColorsChanged?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // LED灯带颜色变化事件（按灯带分组）
      if (props.handlers?.onLedStripColorsChanged) {
        const unlisten = await adaptiveApi.onEvent('LedStripColorsChanged', (data) => {
          // 移除频繁的颜色变化日志
          setStatus(prev => ({
            ...prev,
            lastMessage: 'LED灯带颜色更新',
            messageCount: prev.messageCount + 1
          }));
          props.handlers?.onLedStripColorsChanged?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // 配置变化事件
      if (props.handlers?.onConfigChanged) {
        const unlisten = await adaptiveApi.onEvent('ConfigChanged', (data) => {
          console.log('⚙️ 配置变化:', data);
          setStatus(prev => ({ 
            ...prev, 
            lastMessage: '配置更新',
            messageCount: prev.messageCount + 1 
          }));
          props.handlers?.onConfigChanged?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // 环境光状态变化事件
      if (props.handlers?.onAmbientLightStateChanged) {
        const unlisten = await adaptiveApi.onEvent('AmbientLightStateChanged', (data) => {
          if (import.meta.env.DEV) {
            console.log('环境光状态变化:', data);
          }
          setStatus(prev => ({
            ...prev,
            lastMessage: '环境光状态更新',
            messageCount: prev.messageCount + 1
          }));
          props.handlers?.onAmbientLightStateChanged?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // 设备列表变化事件
      if (props.handlers?.onBoardsChanged) {
        const unlisten = await adaptiveApi.onEvent('BoardsChanged', (data) => {
          if (import.meta.env.DEV) {
            console.log('设备列表变化:', data);
          }
          setStatus(prev => ({
            ...prev,
            lastMessage: '设备列表更新',
            messageCount: prev.messageCount + 1
          }));
          props.handlers?.onBoardsChanged?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // 显示器状态变化事件
      if (props.handlers?.onDisplaysChanged) {
        const unlisten = await adaptiveApi.onEvent('DisplaysChanged', (data) => {
          if (import.meta.env.DEV) {
            console.log('显示器状态变化:', data);
          }
          setStatus(prev => ({
            ...prev,
            lastMessage: '显示器状态更新',
            messageCount: prev.messageCount + 1
          }));
          props.handlers?.onDisplaysChanged?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // 导航事件
      if (props.handlers?.onNavigate) {
        const unlisten = await adaptiveApi.onEvent('Navigate', (data) => {
          if (import.meta.env.DEV) {
            console.log('导航事件:', data);
          }
          setStatus(prev => ({
            ...prev,
            lastMessage: '导航更新',
            messageCount: prev.messageCount + 1
          }));
          props.handlers?.onNavigate?.(data);
        });
        unlistenFunctions.push(unlisten);
      }

      // 只在开发模式下监听所有事件（用于调试）
      if (import.meta.env.DEV) {
        const unlistenAll = await adaptiveApi.onEvent('*', (message) => {
          console.log('WebSocket消息:', message);
        });
        unlistenFunctions.push(unlistenAll);
      }

      updateConnectionStatus(true);
      if (import.meta.env.DEV) {
        console.log('WebSocket事件监听器已注册');
      }

    } catch (error) {
      console.error('❌ 注册WebSocket事件监听器失败:', error);
      updateConnectionStatus(false);
    }
  };

  // 清理事件监听器
  const cleanup = () => {
    unlistenFunctions.forEach(unlisten => {
      try {
        unlisten();
      } catch (error) {
        console.warn('清理事件监听器时出错:', error);
      }
    });
    unlistenFunctions.length = 0;
    updateConnectionStatus(false);
  };

  // 组件挂载时注册监听器
  onMount(() => {
    if (props.autoConnect !== false) {
      registerEventListeners();
    }
  });

  // 组件卸载时清理
  onCleanup(() => {
    cleanup();
  });

  // 手动连接/断开连接
  const connect = () => registerEventListeners();
  const disconnect = () => cleanup();

  // 如果不显示状态，返回空
  if (!props.showStatus) {
    return null;
  }

  // 渲染连接状态
  return (
    <div class="websocket-status">
      <div class={`status-indicator ${status().connected ? 'connected' : 'disconnected'}`}>
        <span class="status-dot"></span>
        <span class="status-text">
          {status().connected ? '已连接' : '未连接'}
        </span>
      </div>
      
      {status().connected && (
        <div class="status-details">
          <span class="message-count">消息: {status().messageCount}</span>
          {status().lastMessage && (
            <span class="last-message">最新: {status().lastMessage}</span>
          )}
        </div>
      )}

      <div class="status-controls">
        <button 
          onClick={status().connected ? disconnect : connect}
          class={`btn btn-sm ${status().connected ? 'btn-warning' : 'btn-primary'}`}
        >
          {status().connected ? '断开' : '连接'}
        </button>
      </div>

      <style>{`
        .websocket-status {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 0.5rem;
          background: var(--fallback-b2, oklch(var(--b2)));
          border-radius: 0.5rem;
          font-size: 0.875rem;
        }

        .status-indicator {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .status-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          background: var(--fallback-er, oklch(var(--er)));
        }

        .connected .status-dot {
          background: var(--fallback-su, oklch(var(--su)));
        }

        .status-details {
          display: flex;
          gap: 1rem;
          font-size: 0.75rem;
          opacity: 0.7;
        }

        .status-controls {
          margin-left: auto;
        }
      `}</style>
    </div>
  );
};

// 导出便捷的Hook函数
export const useWebSocketEvents = (handlers: WebSocketEventHandlers) => {
  onMount(async () => {
    try {
      await adaptiveApi.initialize();
      
      const unlistenFunctions: (() => void)[] = [];

      // 注册所有处理器
      for (const [eventType, handler] of Object.entries(handlers)) {
        if (handler && typeof handler === 'function') {
          const eventName = eventType.replace(/^on/, '').replace(/Changed$/, '');
          const unlisten = await adaptiveApi.onEvent(eventName, handler);
          unlistenFunctions.push(unlisten);
        }
      }

      // 清理函数
      onCleanup(() => {
        unlistenFunctions.forEach(unlisten => unlisten());
      });

    } catch (error) {
      console.error('注册WebSocket事件处理器失败:', error);
    }
  });
};
