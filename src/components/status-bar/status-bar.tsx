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
import { LedPreview } from '../led-preview';

export interface StatusBarProps {
  class?: string;
  compact?: boolean; // 紧凑模式，显示更少信息
}

interface LedPreviewStateChangedEvent {
  state: {
    enabled: boolean;
  };
}

export function StatusBar(props: StatusBarProps) {
  const { t } = useLanguage();
  const [statusData, setStatusData] = createSignal<StatusBarData | null>(null);
  const [connected, setConnected] = createSignal(false);
  const [lastMessageTime, setLastMessageTime] = createSignal<Date | null>(null);
  const [ledPreviewEnabled, setLedPreviewEnabled] = createSignal(false);

  // WebSocket连接状态监听
  let unsubscribeStatus: (() => void) | null = null;
  let unsubscribeConnection: (() => void) | null = null;
  let unsubscribeLedPreview: (() => void) | null = null;

  onMount(async () => {
    try {
      if (import.meta.env.DEV) {
        console.log('Status bar initializing...');
      }

      // 初始化时主动获取一次状态
      try {
        console.log('🔄 Fetching initial LED status...');
        const initialMode = await adaptiveApi.getDataSendMode();
        console.log('📊 Initial LED mode:', initialMode);

        // 创建一个模拟的WebSocket事件来初始化状态
        const mockEvent = {
          status: {
            mode: initialMode,
            frequency: initialMode === 'AmbientLight' ? 30 : (initialMode === 'None' ? 0 : 1),
            data_length: 0,
            total_led_count: 0,
            test_mode_active: initialMode === 'TestEffect',
            timestamp: new Date().toISOString()
          }
        };

        const statusBarData = convertToStatusBarData(mockEvent.status, true, t);
        console.log('📊 Initial status bar data:', statusBarData);
        setStatusData(statusBarData);
        setConnected(true);
        console.log('✅ Initial status loaded successfully');
      } catch (error) {
        console.error('❌ Failed to fetch initial status:', error);
      }

      // 监听LED状态变化事件
      unsubscribeStatus = await adaptiveApi.onEvent<any>(
        'LedStatusChanged',
        (statusData) => {
          // api-adapter.ts 已经提取了 message.data，所以这里直接使用 statusData
          if (statusData && typeof statusData === 'object') {
            try {
              const statusBarData = convertToStatusBarData(statusData, connected(), t);
              console.log(`📊 [${new Date().toISOString()}] Status bar received mode: ${statusBarData.raw_mode}, test_mode_active: ${statusBarData.test_mode_active}`);
              setStatusData(statusBarData);
              setLastMessageTime(new Date());
              // 移除频繁的状态更新日志
            } catch (error) {
              console.error('Error converting status data:', error);
              if (import.meta.env.DEV) {
                console.log('Raw status data:', statusData);
              }
            }
          } else {
            console.warn('Invalid LED status event received:', statusData);
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

      // 监听LED预览状态变化事件
      unsubscribeLedPreview = await adaptiveApi.onEvent<LedPreviewStateChangedEvent>(
        'LedPreviewStateChanged',
        (event) => {
          console.log('🎨 Status bar received LED preview state update:', event);
          if (event && event.state) {
            setLedPreviewEnabled(event.state.enabled);
            console.log('✅ LED preview state updated:', event.state.enabled);
          }
        }
      );

      // 初始化LED预览状态
      try {
        const previewState = await adaptiveApi.getLedPreviewState();
        setLedPreviewEnabled(previewState.enabled);
        console.log('🎨 Initial LED preview state loaded:', previewState.enabled);
      } catch (error) {
        console.error('❌ Failed to load initial LED preview state:', error);
        // 如果获取失败，使用默认值true（因为后端默认已改为true）
        setLedPreviewEnabled(true);
      }

      // 订阅LED状态变化事件
      console.log('📤 Subscribing to events...');
      try {
        await adaptiveApi.subscribeToEvents(['LedStatusChanged', 'LedPreviewStateChanged']);
        console.log('✅ Subscribed to events');
      } catch (subscribeError) {
        console.error('❌ Failed to subscribe to events:', subscribeError);
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
    if (unsubscribeLedPreview) {
      unsubscribeLedPreview();
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
    <div class={`space-y-1 ${props.class || ''}`}>
      <div class="flex items-center gap-2 px-3 py-1 bg-base-200 rounded-lg text-sm">
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

        {/* LED预览切换按钮 */}
        <div class="ml-auto">
          <button
            class={`btn btn-xs ${ledPreviewEnabled() ? 'btn-primary' : 'btn-ghost'}`}
            onClick={toggleLedPreview}
            title={ledPreviewEnabled() ? t('tray.ledPreviewEnabled') : t('tray.ledPreviewDisabled')}
          >
            🎨
          </button>
        </div>
      </div>
    </div>
  );

  // 切换LED预览状态
  const toggleLedPreview = async () => {
    try {
      const newState = !ledPreviewEnabled();

      // 调用API来保存状态到后端
      await adaptiveApi.setLedPreviewState(newState);

      // 只有在API调用成功后才更新本地状态
      // 实际状态会通过WebSocket事件更新，但为了即时响应也在这里更新
      setLedPreviewEnabled(newState);

      console.log('LED preview toggled to:', newState);
    } catch (error) {
      console.error('Failed to toggle LED preview:', error);
      // 如果API调用失败，恢复原状态
      // setLedPreviewEnabled(!newState); // 不需要，因为上面没有更新状态
    }
  };

  // 完整模式渲染 - 优化为一行显示
  const renderFull = () => (
    <div class={`space-y-2 ${props.class || ''}`}>
      {/* 状态栏 */}
      <div class="bg-base-100 border border-base-300 rounded-lg px-4 py-2">
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

              {/* LED预览切换按钮 */}
              <div class="ml-auto">
                <button
                  class={`btn btn-xs ${ledPreviewEnabled() ? 'btn-primary' : 'btn-ghost'}`}
                  onClick={toggleLedPreview}
                  title={ledPreviewEnabled() ? t('tray.ledPreviewEnabled') : t('tray.ledPreviewDisabled')}
                >
                  🎨 {t('tray.ledPreview')}
                </button>
              </div>
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

              {/* LED预览切换按钮 */}
              <div class="ml-auto">
                <button
                  class={`btn btn-xs ${ledPreviewEnabled() ? 'btn-primary' : 'btn-ghost'}`}
                  onClick={toggleLedPreview}
                  title={ledPreviewEnabled() ? t('tray.ledPreviewEnabled') : t('tray.ledPreviewDisabled')}
                >
                  🎨 {t('tray.ledPreview')}
                </button>
              </div>
            </div>
          )}
        </Show>
      </div>

      {/* LED预览 */}
      <Show when={ledPreviewEnabled()}>
        <LedPreview maxLeds={200} />
      </Show>
    </div>
  );

  return props.compact ? renderCompact() : renderFull();
}
