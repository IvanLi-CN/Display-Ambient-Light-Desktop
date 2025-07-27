/**
 * LED灯带预览组件
 * 订阅排序后的LED颜色数据，以一行的形式显示所有LED的颜色
 */

import { createSignal, onMount, onCleanup, Show, For } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import { useLanguage } from '../../i18n/index';
import { DataSendMode } from '../../types/led-status';
import { LedSortedColorsChangedEvent } from '../../types/websocket';

export interface LedPreviewProps {
  class?: string;
  maxLeds?: number; // 最大显示的LED数量，超过则缩放
}

export function LedPreview(props: LedPreviewProps) {
  const { t } = useLanguage();
  const [sortedColors, setSortedColors] = createSignal<Uint8ClampedArray>(new Uint8ClampedArray(0));
  const [connected, setConnected] = createSignal(false);
  const [lastUpdateTime, setLastUpdateTime] = createSignal<Date | null>(null);

  // 用于组装分片数据的缓冲区
  const [colorBuffer, setColorBuffer] = createSignal<Map<number, Uint8ClampedArray>>(new Map());

  let unsubscribeSortedColors: (() => void) | null = null;
  let unsubscribeConnection: (() => void) | null = null;

  // 防抖动相关变量
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let lastDataSize = 0;
  let stableDataCount = 0;
  const DEBOUNCE_DELAY = 100; // 100ms防抖
  const STABLE_COUNT_THRESHOLD = 3; // 需要连续3次相同大小才认为稳定

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
  const updateColors = (event: LedSortedColorsChangedEvent) => {
    const colorsArray = new Uint8ClampedArray(event.sorted_colors);
    const ledOffset = event.led_offset || 0; // 向后兼容，默认偏移量为0

    // 将LED偏移量转换为字节偏移量（每个LED占3字节RGB）
    const byteOffset = ledOffset * 3;

    console.log('🌈 LED Preview received fragment:', {
      bytes: colorsArray.length,
      ledOffset: ledOffset,
      byteOffset: byteOffset,
      mode: event.mode
    });

    // 更新缓冲区中的分片数据（使用字节偏移量作为key）
    const currentBuffer = new Map(colorBuffer());
    currentBuffer.set(byteOffset, colorsArray);
    setColorBuffer(currentBuffer);

    // 组装完整的LED数据
    const assembledColors = assembleColorFragments(currentBuffer);

    // Apply maxLeds limit if specified
    const maxBytes = props.maxLeds ? props.maxLeds * 3 : assembledColors.length;
    const limitedColors = assembledColors.slice(0, maxBytes);

    setSortedColors(limitedColors);
    setLastUpdateTime(new Date());
    console.log('✅ LED Preview colors updated:', limitedColors.length, 'bytes, mode:', event.mode);
  };

  onMount(async () => {
    try {
      console.log('🎨 LED Preview initializing...');

      // 监听LED排序颜色变化事件
      unsubscribeSortedColors = await adaptiveApi.onEvent<LedSortedColorsChangedEvent>(
        'LedSortedColorsChanged',
        (event) => {
          console.log('🌈 LED Preview received sorted colors update:', event);

          if (event && event.sorted_colors) {
            try {
              // 检查模式，只在特定模式下更新预览
              const mode = event.mode || 'AmbientLight'; // 默认为氛围光模式以保持向后兼容

              // 只在氛围光模式、测试模式或灯带配置模式下更新LED预览
              if (mode === 'AmbientLight' || mode === 'TestEffect' || mode === 'StripConfig') {
                const currentDataSize = event.sorted_colors.length;

                // 检查数据大小稳定性
                if (currentDataSize === lastDataSize) {
                  stableDataCount++;
                } else {
                  stableDataCount = 1;
                  lastDataSize = currentDataSize;
                }

                // 清除之前的防抖定时器
                if (debounceTimer) {
                  clearTimeout(debounceTimer);
                }

                // 只有在数据稳定或者是第一次更新时才立即更新
                if (stableDataCount >= STABLE_COUNT_THRESHOLD || sortedColors().length === 0) {
                  updateColors(event);
                } else {
                  // 否则使用防抖延迟更新
                  debounceTimer = setTimeout(() => {
                    updateColors(event);
                  }, DEBOUNCE_DELAY);
                }
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

      // 订阅LED排序颜色变化事件
      console.log('📤 Subscribing to LedSortedColorsChanged events...');
      try {
        await adaptiveApi.subscribeToEvents(['LedSortedColorsChanged']);
        console.log('✅ Subscribed to LedSortedColorsChanged events');
      } catch (subscribeError) {
        console.error('❌ Failed to subscribe to LedSortedColorsChanged events:', subscribeError);
      }

      // 设置连接状态为true（假设WebSocket已连接）
      setConnected(true);

      console.log('✅ LED Preview WebSocket listeners initialized');

    } catch (error) {
      console.error('❌ Failed to initialize LED Preview WebSocket listeners:', error);
    }
  });

  onCleanup(() => {
    // 清理防抖定时器
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }

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

    // 后端发送的数据已经是RGB格式，直接解析
    for (let i = 0; i < colors.length; i += 3) {
      if (i + 2 < colors.length) {
        const r = colors[i];     // Red
        const g = colors[i + 1]; // Green
        const b = colors[i + 2]; // Blue
        ledColors.push(`rgb(${r}, ${g}, ${b})`);
      }
    }

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

  const displayInfo = () => getDisplayInfo();

  return (
    <div class={`bg-base-100 border border-base-300 rounded-lg px-3 py-2 ${props.class || ''}`}>
      <div class="flex items-center gap-2 mb-2">
        {/* 连接状态指示器 */}
        <div
          class="w-2 h-2 rounded-full flex-shrink-0"
          style={{ 'background-color': getConnectionColor() }}
          title={getConnectionText()}
        />
        
        {/* 标题 */}
        <span class="text-sm font-medium text-base-content/80">
          {t('tray.ledPreview')}
        </span>
        
        {/* LED数量 */}
        <Show when={displayInfo().showCount > 0}>
          <span class="text-xs text-base-content/60">
            ({displayInfo().showCount} LEDs)
          </span>
        </Show>
        
        {/* 最后更新时间 */}
        <Show when={lastUpdateTime()}>
          <span class="text-xs text-base-content/40 ml-auto">
            {lastUpdateTime()?.toLocaleTimeString()}
          </span>
        </Show>
      </div>
      
      {/* LED颜色显示 */}
      <Show
        when={displayInfo().colors.length > 0}
        fallback={
          <div class="flex items-center justify-center h-6 text-base-content/60 text-xs">
            {connected() ? t('ledStatus.waitingForData') : t('ledStatus.disconnected')}
          </div>
        }
      >
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
      </Show>
    </div>
  );
}
