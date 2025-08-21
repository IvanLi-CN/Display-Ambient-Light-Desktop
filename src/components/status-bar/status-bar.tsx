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

  // 频率统计（滑动窗口 + EMA 平滑 + 空闲超时）
  const frequencyWindowSize = 20;
  const idleTimeoutMs = 3000; // 空闲超时：3s
  const emaAlpha = 0.2; // EMA 平滑因子
  const minWindowDurationMs = 500; // 计算窗口至少覆盖500ms以避免噪声
  const timestampHistory: number[] = [];
  let emaFrequency = 0; // 指数滑动平均频率
  let lastReceiveMs = 0;
  let idleResetTimer: ReturnType<typeof setTimeout> | null = null;

  const computeWindowFrequencyHz = () => {
    if (timestampHistory.length < 2) return 0;
    const first = timestampHistory[0];
    const last = timestampHistory[timestampHistory.length - 1];
    const durationMs = last - first;
    if (durationMs <= 0 || durationMs < minWindowDurationMs) return 0;
    const intervals = timestampHistory.length - 1;
    const hz = (intervals * 1000) / durationMs;
    return Math.round(hz * 10) / 10; // 保留1位小数
  };

  const applyEma = (value: number) => {
    if (emaFrequency === 0) {
      emaFrequency = value; // 初次赋值
    } else {
      emaFrequency = Math.round((emaAlpha * value + (1 - emaAlpha) * emaFrequency) * 10) / 10;
    }
    return emaFrequency;
  };

  const resetFrequencyStats = () => {
    timestampHistory.length = 0;
    emaFrequency = 0;
    lastReceiveMs = 0;
    if (idleResetTimer) {
      clearTimeout(idleResetTimer);
      idleResetTimer = null;
    }
    // 立即更新 UI 的频率为 0
    const current = statusData();
    if (current) setStatusData({ ...current, frequency: 0 });
  };

  const scheduleIdleReset = () => {
    if (idleResetTimer) clearTimeout(idleResetTimer);
    idleResetTimer = setTimeout(() => {
      // 如果超过 idleTimeoutMs 未收到新消息，则重置频率
      const now = Date.now();
      if (lastReceiveMs && now - lastReceiveMs >= idleTimeoutMs) {
        resetFrequencyStats();
      }
    }, idleTimeoutMs + 50);
  };

  // WebSocket连接状态监听
  let unsubscribeStatus: (() => void) | null = null;
  let unsubscribeConnection: (() => void) | null = null;
  let unsubscribeLedPreview: (() => void) | null = null;
  let unsubscribeSortedColors: (() => void) | null = null;

  onMount(async () => {
    try {
      if (import.meta.env.DEV) {
        console.log('Status bar initializing...');
      }

      // 初始化时主动获取一次状态
      try {
        console.log('🔄 Fetching initial LED status...');
        const [initialMode, ledStatus] = await Promise.all([
          adaptiveApi.getDataSendMode(),
          adaptiveApi.getLedStatus()
        ]);
        console.log('📊 Initial LED mode:', initialMode);
        console.log('📊 Initial LED status:', ledStatus);

        // 使用真实的LED状态数据（频率初始为0，等待实时计算）
        const statusEvent = {
          data_send_mode: initialMode,
          frequency: 0,
          data_length: ledStatus.current_colors_bytes || 0,
          total_led_count: Math.floor((ledStatus.current_colors_bytes || 0) / 3), // 假设RGB，每个LED 3字节
          test_mode_active: initialMode === 'TestEffect',
          timestamp: new Date().toISOString()
        };

        const statusBarData = convertToStatusBarData(statusEvent, true, t);
        console.log('📊 Initial status bar data:', statusBarData);
        setStatusData(statusBarData);
        setConnected(true);
        console.log('✅ Initial status loaded successfully');
      } catch (error) {
        console.error('❌ Failed to fetch initial status:', error);
      }

      // 监听LED状态变化事件（用于频率/模式/连接）
      unsubscribeStatus = await adaptiveApi.onEvent<any>(
        'LedStatusChanged',
        (statusData) => {
          // api-adapter.ts 已经提取了 message.data，所以这里直接使用 statusData
          if (statusData && typeof statusData === 'object') {
            try {
              // 如果模式为 None，重置频率并直接更新 UI
              const mode = (statusData.data_send_mode || statusData.mode) as DataSendMode | undefined;
              if (mode === 'None') {
                resetFrequencyStats();
              }

              // 更新本地频率统计
              const now = Date.now();
              lastReceiveMs = now;
              timestampHistory.push(now);
              if (timestampHistory.length > frequencyWindowSize) {
                timestampHistory.shift();
              }

              // 计算窗口频率并应用 EMA 平滑
              const windowHz = computeWindowFrequencyHz();
              const realtimeHz = applyEma(windowHz);
              scheduleIdleReset();

              const statusBarData = convertToStatusBarData(statusData, connected(), t);
              const updated: StatusBarData = { ...statusBarData, frequency: realtimeHz };
              // 日志：频率 + 模式
              if (import.meta.env.DEV) {
                console.log(`📊 [${new Date().toISOString()}] Status mode=${updated.raw_mode}, test=${updated.test_mode_active}, windowHz=${windowHz}, emaHz=${updated.frequency}`);
              }
              setStatusData(updated);
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

          // 断开连接时重置频率统计
          if (!isConnected) {
            resetFrequencyStats();
          }

          // 更新现有状态数据的连接状态
          const current = statusData();
          if (current) {
            setStatusData({ ...current, connected: isConnected, frequency: isConnected ? current.frequency : 0 });
          }
        }
      );

      // 监听 LED 排序颜色变化事件（也可触发频率统计，避免仅依赖 LedStatusChanged 的发送频率）
      try {
        unsubscribeSortedColors = await adaptiveApi.onEvent<any>(
          'LedSortedColorsChanged',
          () => {
            const now = Date.now();
            lastReceiveMs = now;
            timestampHistory.push(now);
            if (timestampHistory.length > frequencyWindowSize) timestampHistory.shift();
            const windowHz = computeWindowFrequencyHz();
            const realtimeHz = applyEma(windowHz);
            scheduleIdleReset();

            const current = statusData();
            if (current) {
              setStatusData({ ...current, frequency: realtimeHz });
            }
          }
        );
      } catch (error) {
        console.error('❌ Failed to listen LedSortedColorsChanged for frequency:', error);
      }

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
    if (unsubscribeSortedColors) {
      unsubscribeSortedColors();
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

  // 检查是否有有效的上次更新时间
  const hasValidLastMessageTime = () => {
    const d = lastMessageTime();
    return d instanceof Date && !isNaN(d.getTime());
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

              {/* LED数量 + 时间（无时间则仅显示无数据） */}
              <span class="text-base-content/60 text-xs flex-shrink-0">
                {hasValidLastMessageTime()
                  ? `${data().total_led_count} LEDs, ${lastMessageTime()!.toLocaleTimeString('zh-CN', { hour12: false })}`
                  : t('common.noData')}
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

              {/* LED数量 + 时间（无时间则仅显示无数据） */}
              <div class="ml-2 text-xs text-base-content/60 flex-shrink-0">
                <span>
                  {hasValidLastMessageTime()
                    ? `${data().total_led_count} LEDs, ${lastMessageTime()!.toLocaleTimeString('zh-CN', { hour12: false })}`
                    : t('common.noData')}
                </span>
              </div>



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
      <LedPreview maxLeds={200} enabled={ledPreviewEnabled()} />
    </div>
  );

  return props.compact ? renderCompact() : renderFull();
}
