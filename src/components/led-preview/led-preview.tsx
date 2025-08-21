/**
 * LED灯带预览组件
 * 订阅排序后的LED颜色数据，以一行的形式显示所有LED的颜色
 */

import { createSignal, createMemo, onMount, onCleanup, Show, For } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import { useLanguage } from '../../i18n/index';
import { DataSendMode } from '../../types/led-status';
import { LedSortedColorsChangedEvent } from '../../types/websocket';
import { LedApiService } from '../../services/led-api.service';

export interface LedPreviewProps {
  class?: string;
  maxLeds?: number; // 最大显示的LED数量，超过则缩放
  enabled?: boolean; // 是否启用LED预览
}

export function LedPreview(props: LedPreviewProps) {
  const { t } = useLanguage();
  const [sortedColors, setSortedColors] = createSignal<Uint8ClampedArray>(new Uint8ClampedArray(0));
  const [connected, setConnected] = createSignal(false);
  const [lastUpdateTime, setLastUpdateTime] = createSignal<Date | null>(null);


  // 用于组装分片数据的缓冲区
  const [colorBuffer, setColorBuffer] = createSignal<Map<number, Uint8ClampedArray>>(new Map());
  // 记录当前模式，用于检测模式切换
  const [currentMode, setCurrentMode] = createSignal<string>('');

  let unsubscribeSortedColors: (() => void) | null = null;
  let unsubscribeConnection: (() => void) | null = null;

  // 渲染节流相关变量（目标 30FPS）
  let renderTimer: ReturnType<typeof setTimeout> | null = null;
  let lastRenderMs = 0;
  const TARGET_FPS = 30;
  const MIN_RENDER_INTERVAL = Math.floor(1000 / TARGET_FPS);
  let pendingEvent: LedSortedColorsChangedEvent | null = null;

  const scheduleRender = (event: LedSortedColorsChangedEvent, fromPolling = false) => {
    pendingEvent = event;
    const now = Date.now();
    const elapsed = now - lastRenderMs;

    const doRender = () => {
      if (pendingEvent) {
        updateColors(pendingEvent, fromPolling);
        pendingEvent = null;
      }
      lastRenderMs = Date.now();
      renderTimer = null;
    };

    if (elapsed >= MIN_RENDER_INTERVAL) {
      doRender();
    } else if (!renderTimer) {
      renderTimer = setTimeout(doRender, MIN_RENDER_INTERVAL - elapsed);
    }
  };

  // 轮询相关变量（在预览界面加速，提升可见刷新率）
  let pollingTimer: ReturnType<typeof setInterval> | null = null;
  let lastWebSocketUpdate = Date.now();
  const POLLING_INTERVAL = 200; // 200ms 轮询间隔（5Hz）
  const WEBSOCKET_TIMEOUT = 500; // 500ms 无WebSocket数据则开始轮询

  // 轮询获取LED颜色数据（用于氛围光模式）
  const pollLedColors = async () => {
    try {
      console.log('🔄 Polling LED colors from API...');

      // 🔧 同时获取LED颜色数据和状态信息（包含真实时间戳）
      const [colors, ledStatus] = await Promise.all([
        LedApiService.getCurrentLedColors(),
        adaptiveApi.getLedStatus()
      ]);

      if (colors && colors.length > 0) {
        console.log('🌈 Polled LED colors:', colors.length, 'bytes');

        // 模拟WebSocket事件格式
        const mockEvent = {
          sorted_colors: colors,
          mode: 'AmbientLight' as DataSendMode,
          led_offset: 0
        };

        // 走统一的节流渲染通道
        scheduleRender(mockEvent, true);

        // 🔧 使用后端状态中的真实时间戳
        if (ledStatus && ledStatus.last_updated) {
          setLastUpdateTime(new Date(ledStatus.last_updated));
          // console.log('🕒 Updated timestamp from backend:', ledStatus.last_updated);
        }
      } else {
        console.log('📭 No LED color data available from API');
      }
    } catch (error) {
      console.error('❌ Failed to poll LED colors:', error);
    }
  };

  // 启动轮询机制
  const startPolling = () => {
    if (pollingTimer) {
      clearInterval(pollingTimer);
    }

    console.log('🔄 Starting LED color polling...');
    pollingTimer = setInterval(() => {
      const timeSinceLastUpdate = Date.now() - lastWebSocketUpdate;

      // 如果超过WEBSOCKET_TIMEOUT时间没有收到WebSocket数据，则开始轮询
      if (timeSinceLastUpdate > WEBSOCKET_TIMEOUT) {
        pollLedColors();
      }
    }, POLLING_INTERVAL);
  };

  // 停止轮询机制
  const stopPolling = () => {
    if (pollingTimer) {
      console.log('⏹️ Stopping LED color polling...');
      clearInterval(pollingTimer);
      pollingTimer = null;
    }
  };

  // 组装颜色分片为完整数据
  const assembleColorFragments = (buffer: Map<number, Uint8ClampedArray>): Uint8ClampedArray => {
    if (buffer.size === 0) {
      return new Uint8ClampedArray();
    }

    // 按偏移量排序分片
    const sortedFragments = Array.from(buffer.entries()).sort(([a], [b]) => a - b);

    // 计算总长度 - 找到最大的结束位置
    let totalLength = 0;
    for (const [offset, fragment] of sortedFragments) {
      const endPosition = offset + fragment.length;
      totalLength = Math.max(totalLength, endPosition);
    }

    // 容错：如果后端分片未从0开始，确保数组足够大
    if (sortedFragments.length > 0 && sortedFragments[0][0] > 0) {
      totalLength = Math.max(totalLength, sortedFragments[0][0] + sortedFragments[0][1].length);
    }

    // 创建完整的颜色数组
    const assembledArray = new Uint8ClampedArray(totalLength);

    // 填充分片数据
    for (const [offset, fragment] of sortedFragments) {
      assembledArray.set(fragment, offset);
    }

    console.log('🔧 Assembled LED data:', {
      fragments: sortedFragments.length,
      totalBytes: totalLength,
      fragmentSizes: sortedFragments.map(([offset, fragment]) => `${offset}:${fragment.length}`),
      fragmentDetails: sortedFragments.map(([offset, fragment]) => `offset=${offset}, length=${fragment.length}, end=${offset + fragment.length}`)
    });

    return assembledArray;
  };

  // 颜色更新函数 - 处理分片数据
  const updateColors = (event: LedSortedColorsChangedEvent, fromPolling = false) => {
    const colorsArray = new Uint8ClampedArray(event.sorted_colors);
    const ledOffset = event.led_offset || 0; // 向后兼容，默认偏移量为0
    const mode = event.mode || 'AmbientLight';

    // 如果不是来自轮询，则更新WebSocket数据时间戳
    if (!fromPolling) {
      lastWebSocketUpdate = Date.now();

      // 🔧 使用WebSocket事件中的时间戳（如果有的话）
      if (event.timestamp) {
        setLastUpdateTime(new Date(event.timestamp));
        // console.log('🕒 Updated timestamp from WebSocket event:', event.timestamp);
      }
    }

    // 将LED偏移量转换为字节偏移量（每个LED占3字节RGB）
    const byteOffset = ledOffset * 3;

    console.log('🌈 LED Preview received fragment:', {
      bytes: colorsArray.length,
      ledOffset: ledOffset,
      byteOffset: byteOffset,
      mode: mode,
      firstFewBytes: colorsArray.length > 0 ? Array.from(colorsArray.slice(0, 12)) : 'empty'
    });

    // 检测模式切换，如果模式改变则清理缓冲区
    if (currentMode() !== mode) {
      console.log('🔄 LED Preview mode changed from', currentMode(), 'to', mode, '- clearing buffer');
      setColorBuffer(new Map());
      setCurrentMode(mode);
    }

    // 更新缓冲区中的分片数据（使用字节偏移量作为key）
    const currentBuffer = new Map(colorBuffer());
    currentBuffer.set(byteOffset, colorsArray);
    setColorBuffer(currentBuffer);

    // 组装完整的LED数据
    const assembledColors = assembleColorFragments(currentBuffer);

    // Apply maxLeds limit if specified
    const maxBytes = props.maxLeds ? props.maxLeds * 3 : assembledColors.length;
    const limitedColors = assembledColors.slice(0, maxBytes);

    console.log('🎨 Before setSortedColors:', {
      assembledLength: assembledColors.length,
      limitedLength: limitedColors.length,
      currentSortedLength: sortedColors().length,
      firstFewAssembled: assembledColors.length > 0 ? Array.from(assembledColors.slice(0, 12)) : 'empty',
      firstFewLimited: limitedColors.length > 0 ? Array.from(limitedColors.slice(0, 12)) : 'empty'
    });

    setSortedColors(limitedColors);
    // 🔧 移除前端自己生成时间戳，应该从后端数据中获取
    // setLastUpdateTime(new Date());

    console.log('🎨 After setSortedColors:', {
      newSortedLength: limitedColors.length,
      firstFewSorted: limitedColors.length > 0 ? Array.from(limitedColors.slice(0, 12)) : 'empty'
    });

    console.log('✅ LED Preview colors updated:', limitedColors.length, 'bytes, mode:', event.mode);
  };

  onMount(async () => {
    try {
      console.log('🎨 LED Preview initializing...');
      console.log('🎨 LED Preview enabled:', props.enabled);



      // 监听LED排序颜色变化事件
      console.log('📤 Subscribing to LedSortedColorsChanged events...');
      unsubscribeSortedColors = await adaptiveApi.onEvent<LedSortedColorsChangedEvent>(
        'LedSortedColorsChanged',
        (event) => {
          console.log('🌈 LED Preview received sorted colors update:', event);

          if (event && event.sorted_colors) {
            try {
              // 检查模式，只在特定模式下更新预览
              const mode = event.mode || 'AmbientLight'; // 默认为氛围光模式以保持向后兼容

              // 只在氛围光模式、测试模式、灯带配置模式或颜色校准模式下更新LED预览
              if (mode === 'AmbientLight' || mode === 'TestEffect' || mode === 'StripConfig' || mode === 'ColorCalibration') {
                const currentDataSize = event.sorted_colors.length;

                // 节流渲染：统一通过 scheduleRender 以 ~30FPS 刷新
                scheduleRender(event);
              } else {
                console.log('🚫 Skipping LED Preview update for mode:', mode);
              }
            } catch (error) {
              console.error('❌ Error processing sorted colors:', error);
            }
          } else {
            console.warn('⚠️ Invalid sorted colors event received:', event);
          }
        }
      );

      // 监听WebSocket连接状态变化
      unsubscribeConnection = await adaptiveApi.onEvent<boolean>(
        'ConnectionStatusChanged',
        (isConnected) => {
          console.log('🔌 LED Preview connection status changed:', isConnected);
          setConnected(isConnected);
        }
      );

      console.log('✅ Subscribed to LedSortedColorsChanged events');

      // 设置连接状态为true（假设WebSocket已连接）
      setConnected(true);

      console.log('✅ LED Preview WebSocket listeners initialized');

      // 启动轮询机制（用于氛围光模式下的数据获取）
      startPolling();

    } catch (error) {
      console.error('❌ Failed to initialize LED Preview WebSocket listeners:', error);
    }
  });

  onCleanup(() => {
    // 清理渲染节流定时器
    if (renderTimer) {
      clearTimeout(renderTimer);
      renderTimer = null;
    }

    // 停止轮询机制
    stopPolling();

    if (unsubscribeSortedColors) {
      unsubscribeSortedColors();
    }
    if (unsubscribeConnection) {
      unsubscribeConnection();
    }
  });

  // 将字节数组转换为LED颜色数组
  const getLedColors = () => {
    const colors = sortedColors();
    const ledColors: string[] = [];

    // 添加详细调试信息
    console.log('🎨 getLedColors() called:', {
      colorsLength: colors.length,
      colorsType: colors.constructor.name,
      firstFewBytes: colors.length > 0 ? Array.from(colors.slice(0, 12)) : 'empty',
      lastFewBytes: colors.length > 12 ? Array.from(colors.slice(-12)) : 'not enough data'
    });



    // 后端发送的数据已经是RGB格式，直接解析
    for (let i = 0; i < colors.length; i += 3) {
      if (i + 2 < colors.length) {
        const r = colors[i];     // Red
        const g = colors[i + 1]; // Green
        const b = colors[i + 2]; // Blue
        ledColors.push(`rgb(${r}, ${g}, ${b})`);

        // 记录前几个LED的颜色用于调试
        if (i < 15) { // 前5个LED
          console.log(`🌈 LED ${i/3}: rgb(${r}, ${g}, ${b})`);
        }
      }
    }

    console.log('🎨 getLedColors() result:', {
      totalLeds: ledColors.length,
      expectedLeds: Math.floor(colors.length / 3),
      firstFewColors: ledColors.slice(0, 5),
      lastFewColors: ledColors.length > 5 ? ledColors.slice(-5) : 'not enough colors'
    });

    return ledColors;
  };

  // 获取显示的LED数量和大小
  const getDisplayInfo = () => {
    const ledColors = getLedColors();
    const totalLeds = ledColors.length;
    const maxLeds = props.maxLeds || 200; // 默认最大显示200个LED
    
    if (totalLeds <= maxLeds) {
      return {
        colors: ledColors,
        ledSize: Math.max(4, Math.min(8, 800 / Math.max(totalLeds, 1))), // 4-8px之间
        showCount: totalLeds
      };
    } else {
      // 如果LED数量太多，进行采样显示
      const step = totalLeds / maxLeds;
      const sampledColors: string[] = [];
      for (let i = 0; i < maxLeds; i++) {
        const index = Math.floor(i * step);
        if (index < ledColors.length) {
          sampledColors.push(ledColors[index]);
        }
      }
      return {
        colors: sampledColors,
        ledSize: Math.max(3, 800 / maxLeds), // 最小3px
        showCount: totalLeds
      };
    }
  };

  // 获取连接状态指示器颜色
  const getConnectionColor = () => {
    if (!connected()) return '#ef4444'; // 红色 - 未连接
    if (sortedColors().length === 0) return '#f59e0b'; // 黄色 - 连接但无数据
    return '#10b981'; // 绿色 - 正常
  };

  // 获取连接状态文本
  const getConnectionText = () => {
    if (!connected()) return t('ledStatus.disconnected');
    if (sortedColors().length === 0) return t('ledStatus.waitingForData');
    return t('ledStatus.connected');
  };

  // 格式化时间（只显示时分秒）
  const formatTimeOnly = (date: Date | null) => {
    if (!date) return '无数据';
    return date.toLocaleString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false
    });
  };

  // 格式化最后更新时间
  const formatLastUpdateTime = () => {
    return formatTimeOnly(lastUpdateTime());
  };

  const displayInfo = createMemo(() => getDisplayInfo());

  return (
    <div class={`${props.class || ''}`} style={{ display: props.enabled === false ? 'none' : 'block' }}>
      {/* LED颜色显示 */}
      <Show
        when={displayInfo().colors.length > 0}
        fallback={
          <div class="flex items-center justify-center h-16 text-base-content/60 text-xs bg-base-100 border border-base-300 rounded">
            <div class="opacity-70">等待状态数据...</div>
          </div>
        }
      >
        <div class="space-y-1">
          <div class="flex gap-0.5 overflow-hidden">
            <For each={displayInfo().colors}>
              {(color) => (
                <div
                  class="flex-shrink-0 rounded-sm"
                  style={{
                    'background-color': color,
                    width: `${displayInfo().ledSize}px`,
                    height: '6px',
                    'min-width': '2px'
                  }}
                  title={color}
                />
              )}
            </For>
          </div>

        </div>
      </Show>
    </div>
  );
}
