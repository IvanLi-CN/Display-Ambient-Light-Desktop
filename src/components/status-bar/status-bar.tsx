/**
 * LED状态栏组件
 * 纯WebSocket驱动，不调用任何API
 */

import { createSignal, onMount, onCleanup, Show } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import {
  LedStatusData,
  StatusBarData,
  convertToStatusBarData,
  LedStatusChangedEvent,
  getModeBadgeStyle,
  getModeIcon,
  DataSendMode
} from '../../types/led-status';
import { useLanguage } from '../../i18n/index';

export interface StatusBarProps {
  class?: string;
  compact?: boolean; // 紧凑模式，显示更少信息
}

export function StatusBar(props: StatusBarProps) {
  const { t } = useLanguage();
  const [statusData, setStatusData] = createSignal<StatusBarData | null>(null);
  const [connected, setConnected] = createSignal(false);
  const [lastMessageTime, setLastMessageTime] = createSignal<Date | null>(null);

  // WebSocket连接状态监听
  let unsubscribeStatus: (() => void) | null = null;
  let unsubscribeConnection: (() => void) | null = null;

  onMount(async () => {
    try {
      console.log('🔧 Status bar initializing...');

      // 监听LED状态变化事件
      unsubscribeStatus = await adaptiveApi.onEvent<LedStatusChangedEvent>(
        'LedStatusChanged',
        (event) => {
          console.log('🔄 Status bar received LED status update:', event);
          console.log('🔍 Event status structure:', event?.status);

          if (event && event.status) {
            try {
              const statusBarData = convertToStatusBarData(event.status, connected(), t);
              setStatusData(statusBarData);
              setLastMessageTime(new Date());
              console.log('✅ Status bar data updated:', statusBarData);
            } catch (error) {
              console.error('❌ Error converting status data:', error);
              console.log('🔍 Raw status data:', event.status);
            }
          } else {
            console.warn('⚠️ Invalid LED status event received:', event);
          }
        }
      );

      // 监听WebSocket连接状态变化
      unsubscribeConnection = await adaptiveApi.onEvent<boolean>(
        'ConnectionStatusChanged',
        (isConnected) => {
          console.log('🔌 Status bar connection status changed:', isConnected);
          setConnected(isConnected);

          // 更新现有状态数据的连接状态
          const current = statusData();
          if (current) {
            setStatusData({ ...current, connected: isConnected });
          }
        }
      );

      // 订阅LED状态变化事件
      console.log('📤 Subscribing to LedStatusChanged events...');
      try {
        await adaptiveApi.subscribeToEvents(['LedStatusChanged']);
        console.log('✅ Subscribed to LedStatusChanged events');
      } catch (subscribeError) {
        console.error('❌ Failed to subscribe to LedStatusChanged events:', subscribeError);
      }

      // 设置连接状态为true（假设WebSocket已连接）
      setConnected(true);

      console.log('✅ Status bar WebSocket listeners initialized');

    } catch (error) {
      console.error('❌ Failed to initialize status bar WebSocket listeners:', error);
    }
  });

  onCleanup(() => {
    if (unsubscribeStatus) {
      unsubscribeStatus();
    }
    if (unsubscribeConnection) {
      unsubscribeConnection();
    }
  });

  // 获取连接状态指示器颜色
  const getConnectionColor = () => {
    if (!connected()) return '#ef4444'; // 红色 - 未连接
    if (!statusData()) return '#f59e0b'; // 黄色 - 连接但无数据
    return '#10b981'; // 绿色 - 正常
  };

  // 获取连接状态文本
  const getConnectionText = () => {
    if (!connected()) return t('ledStatus.disconnected');
    if (!statusData()) return t('ledStatus.waitingForData');
    return t('ledStatus.connected');
  };

  // 格式化数据大小
  const formatDataSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
  };

  // 紧凑模式渲染
  const renderCompact = () => (
    <div class={`flex items-center gap-2 px-3 py-1 bg-base-200 rounded-lg text-sm ${props.class || ''}`}>
      {/* 连接状态指示器 */}
      <div class="flex items-center gap-1">
        <div 
          class="w-2 h-2 rounded-full"
          style={{ 'background-color': getConnectionColor() }}
        />
        <span class="text-xs text-base-content/60">{getConnectionText()}</span>
      </div>

      <Show when={statusData()}>
        {(data) => (
          <>
            <div class="w-px h-4 bg-base-300" />
            <div class={`badge badge-sm ${getModeBadgeStyle(data().raw_mode)} gap-1`}>
              <span class="text-xs">{getModeIcon(data().raw_mode)}</span>
              {data().mode}
            </div>
            <Show when={data().frequency > 0}>
              <span class="text-base-content/60">|</span>
              <span class="text-base-content">{data().frequency}Hz</span>
            </Show>
            <Show when={data().test_mode_active}>
              <div class="badge badge-warning badge-xs">{t('ledStatus.testMode')}</div>
            </Show>
          </>
        )}
      </Show>
    </div>
  );

  // 完整模式渲染
  const renderFull = () => (
    <div class={`bg-base-100 border border-base-300 rounded-lg p-3 ${props.class || ''}`}>
      <div class="flex items-center justify-between mb-2">
        <h3 class="text-sm font-medium text-base-content">{t('ledStatus.title')}</h3>
        <div class="flex items-center gap-2">
          <div 
            class="w-2 h-2 rounded-full"
            style={{ 'background-color': getConnectionColor() }}
          />
          <span class="text-xs text-base-content/60">{getConnectionText()}</span>
        </div>
      </div>

      <Show 
        when={statusData()} 
        fallback={
          <div class="text-center text-base-content/60 py-4">
            <div class="text-sm">{t('ledStatus.waitingForData')}</div>
            <Show when={!connected()}>
              <div class="text-xs mt-1">{t('ledStatus.websocketDisconnected')}</div>
            </Show>
          </div>
        }
      >
        {(data) => (
          <div class="space-y-2">
            {/* 主要状态信息 */}
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-base-content/60">{t('ledStatus.mode')}:</span>
                <div class={`badge badge-sm ml-2 ${getModeBadgeStyle(data().raw_mode)} gap-1`}>
                  <span class="text-xs">{getModeIcon(data().raw_mode)}</span>
                  {data().mode}
                </div>
                <Show when={data().test_mode_active}>
                  <div class="badge badge-warning badge-xs ml-2">{t('ledStatus.testMode')}</div>
                </Show>
              </div>
              <div>
                <span class="text-base-content/60">{t('ledStatus.frequency')}:</span>
                <span class="ml-2 text-base-content">{data().frequency}Hz</span>
              </div>
            </div>

            {/* 数据统计 */}
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-base-content/60">{t('ledStatus.data')}:</span>
                <span class="ml-2 text-base-content">{formatDataSize(data().data_length)}</span>
              </div>
              <div>
                <span class="text-base-content/60">{t('ledStatus.led')}:</span>
                <span class="ml-2 text-base-content">{data().total_led_count}</span>
              </div>
            </div>

            {/* 更新时间 */}
            <div class="text-xs text-base-content/60 pt-1 border-t border-base-300">
              {t('ledStatus.update')}: {data().last_update}
              <Show when={lastMessageTime()}>
                <span class="ml-2">
                  ({t('ledStatus.received')}: {lastMessageTime()!.toLocaleTimeString(undefined, {
                    hour12: false,
                    hour: '2-digit',
                    minute: '2-digit',
                    second: '2-digit'
                  })})
                </span>
              </Show>
            </div>
          </div>
        )}
      </Show>
    </div>
  );

  return props.compact ? renderCompact() : renderFull();
}
