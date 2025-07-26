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

  // 紧凑模式渲染 - 极简一行显示
  const renderCompact = () => (
    <div class={`flex items-center gap-2 px-3 py-1 bg-base-200 rounded-lg text-sm ${props.class || ''}`}>
      {/* 连接状态指示器 */}
      <div
        class="w-2 h-2 rounded-full flex-shrink-0"
        style={{ 'background-color': getConnectionColor() }}
        title={getConnectionText()}
      />

      <Show when={statusData()}>
        {(data) => (
          <>
            {/* 模式徽章 */}
            <div class={`badge badge-sm ${getModeBadgeStyle(data().raw_mode)} gap-1 flex-shrink-0`}>
              <span class="text-xs">{getModeIcon(data().raw_mode)}</span>
              {data().mode}
            </div>

            {/* 测试模式标签 */}
            <Show when={data().test_mode_active}>
              <div class="badge badge-warning badge-xs flex-shrink-0">{t('ledStatus.testMode')}</div>
            </Show>

            {/* 频率 */}
            <Show when={data().frequency > 0}>
              <span class="text-base-content/80 flex-shrink-0">{data().frequency}Hz</span>
            </Show>

            {/* LED数量 */}
            <span class="text-base-content/60 text-xs flex-shrink-0">
              {data().total_led_count} LEDs
            </span>
          </>
        )}
      </Show>
    </div>
  );

  // 完整模式渲染 - 优化为一行显示
  const renderFull = () => (
    <div class={`bg-base-100 border border-base-300 rounded-lg px-4 py-2 ${props.class || ''}`}>
      <Show
        when={statusData()}
        fallback={
          <div class="flex items-center gap-3 text-base-content/60">
            <div
              class="w-2 h-2 rounded-full"
              style={{ 'background-color': getConnectionColor() }}
            />
            <span class="text-sm">{t('ledStatus.waitingForData')}</span>
            <Show when={!connected()}>
              <span class="text-xs">({t('ledStatus.websocketDisconnected')})</span>
            </Show>
          </div>
        }
      >
        {(data) => (
          <div class="flex items-center gap-3">
            {/* 连接状态指示器 */}
            <div
              class="w-2 h-2 rounded-full flex-shrink-0"
              style={{ 'background-color': getConnectionColor() }}
              title={getConnectionText()}
            />

            {/* 模式徽章 */}
            <div class={`badge badge-sm ${getModeBadgeStyle(data().raw_mode)} gap-1 flex-shrink-0`}>
              <span class="text-xs">{getModeIcon(data().raw_mode)}</span>
              {data().mode}
            </div>

            {/* 测试模式标签 */}
            <Show when={data().test_mode_active}>
              <div class="badge badge-warning badge-xs flex-shrink-0">{t('ledStatus.testMode')}</div>
            </Show>

            {/* 频率 */}
            <Show when={data().frequency > 0}>
              <span class="text-sm text-base-content/80 flex-shrink-0">
                {data().frequency}Hz
              </span>
            </Show>

            {/* LED数量 */}
            <span class="text-sm text-base-content/80 flex-shrink-0">
              {data().total_led_count} LEDs
            </span>

            {/* 数据大小（仅在有数据时显示） */}
            <Show when={data().data_length > 0}>
              <span class="text-xs text-base-content/60 flex-shrink-0">
                {formatDataSize(data().data_length)}
              </span>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );

  return props.compact ? renderCompact() : renderFull();
}
