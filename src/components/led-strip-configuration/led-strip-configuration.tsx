import { createEffect } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import { WebSocketListener } from '../websocket-listener';
import { DisplayView } from './display-view';
import { DisplayListContainer } from './display-list-container';
import { displayStore, setDisplayStore } from '../../stores/display.store';
import { LedStripConfigContainer } from '../../models/led-strip-config';
import { setLedStripStore } from '../../stores/led-strip.store';
import { LedStripPartsSorter } from './led-strip-parts-sorter';

import { createStore } from 'solid-js/store';
import {
  LedStripConfigurationContext,
  LedStripConfigurationContextType,
} from '../../contexts/led-strip-configuration.context';
import { useLanguage } from '../../i18n/index';
import { LedStripColorsChangedEvent } from '../../types/websocket';


export const LedStripConfiguration = () => {
  const { t } = useLanguage();

  console.log('🔧 LedStripConfiguration component loaded');

  createEffect(() => {
    // 并行加载显示器信息和配置信息
    Promise.all([
      adaptiveApi.listDisplayInfo(),
      adaptiveApi.getDisplayConfigs(),
      adaptiveApi.getConfig()
    ]).then(([displaysStr, displayConfigs, ledConfigs]) => {
      if (import.meta.env.DEV) {
        console.log('🔍 加载配置数据: displays=%d, configs=%d, strips=%d',
          JSON.parse(displaysStr).length,
          displayConfigs?.length || 0,
          ledConfigs?.strips?.length || 0
        );
      }

      // 解析显示器信息
      const parsedDisplays = JSON.parse(displaysStr);

      // 建立显示器ID映射关系
      const displayIdMap = new Map<number, string>(); // 数字ID -> 内部ID
      const internalIdMap = new Map<string, number>(); // 内部ID -> 数字ID

      if (displayConfigs && Array.isArray(displayConfigs)) {
        displayConfigs.forEach((config: any) => {
          if (config.last_system_id && config.internal_id) {
            displayIdMap.set(config.last_system_id, config.internal_id);
            internalIdMap.set(config.internal_id, config.last_system_id);

            if (import.meta.env.DEV) {
              console.log(`🔗 ID映射: ${config.last_system_id} <-> ${config.internal_id} (${config.name})`);
            }
          }
        });
      }

      // 增强显示器信息，添加内部ID
      const enhancedDisplays = parsedDisplays.map((display: any) => ({
        ...display,
        internal_id: displayIdMap.get(display.id),
        name: displayConfigs?.find((config: any) => config.last_system_id === display.id)?.name
      }));

      setDisplayStore({
        displays: enhancedDisplays,
      });

      // 处理LED配置，确保兼容V2格式
      if (ledConfigs && ledConfigs.strips && Array.isArray(ledConfigs.strips)) {
        if (import.meta.env.DEV) {
          console.log('✅ 转换LED配置: %d strips', ledConfigs.strips.length);
        }

        // 转换V2配置为前端兼容格式
        const convertedStrips = ledConfigs.strips.map((strip: any) => ({
          ...strip,
          // 保持原有的display_id字段用于兼容性
          display_id: internalIdMap.get(strip.display_internal_id) || 0,
          // 添加display_internal_id字段
          display_internal_id: strip.display_internal_id
        }));

        setLedStripStore({
          strips: convertedStrips,
          colorCalibration: ledConfigs.color_calibration || {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            w: 1.0
          }
        });
      } else {
        console.warn('⚠️ LED配置数据无效或缺少strips:', ledConfigs);
        setLedStripStore({
          strips: [],
          colorCalibration: {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            w: 1.0
          }
        });
      }
    }).catch((error) => {
      console.error('❌ 加载配置失败:', error);
      // 设置空的配置
      setDisplayStore({ displays: [] });
      setLedStripStore({
        strips: [],
        colorCalibration: {
          r: 1.0,
          g: 1.0,
          b: 1.0,
          w: 1.0
        }
      });
    });
  });

  // WebSocket event handlers
  const webSocketHandlers = {
    onConfigChanged: (data: any) => {
      console.log('🔧 配置变化事件:', data);
      try {
        // 检查数据结构
        let configData: LedStripConfigContainer;

        if (data && data.config) {
          // 如果数据包装在 config 字段中
          configData = data.config as LedStripConfigContainer;
        } else {
          // 直接使用数据
          configData = data as LedStripConfigContainer;
        }

        // 安全检查：确保 strips 存在且是数组
        if (configData && configData.strips && Array.isArray(configData.strips)) {
          console.log('✅ 有效的配置数据，strips数量:', configData.strips.length);
          setLedStripStore({
            strips: configData.strips,
          });
        } else {
          console.warn('⚠️ 配置数据无效或缺少strips:', configData);
          console.warn('数据结构:', typeof configData, Object.keys(configData || {}));
        }
      } catch (error) {
        console.error('❌ 处理配置变化事件失败:', error);
      }
    },
    // 迁移到按灯带分组的颜色事件处理器
    onLedStripColorsChanged: (data: LedStripColorsChangedEvent) => {
      if (!window.document.hidden) {
        console.log('🎨 LED灯带颜色变化事件:', data);

        // 生成灯带唯一键
        const stripKey = `${data.display_id}:${data.border}:${data.strip_index}`;
        const colorsArray = new Uint8ClampedArray(data.colors);

        // 更新按灯带分组的颜色数据
        setLedStripStore('stripColors', (prev) => {
          const newMap = new Map(prev);
          newMap.set(stripKey, colorsArray);
          return newMap;
        });
      }
    },
    // 添加LED排序颜色变化事件处理器
    onLedSortedColorsChanged: (data: any) => {
      if (!window.document.hidden) {
        console.log('🌈 LED排序颜色变化事件:', data);
        // 数据应该是 { sorted_colors: Vec<u8> } 格式
        const sortedColors = data.sorted_colors || data;
        const sortedColorsArray = new Uint8ClampedArray(sortedColors);
        setLedStripStore({
          sortedColors: sortedColorsArray,
        });
      }
    },
  };

  const [ledStripConfiguration, setLedStripConfiguration] = createStore<
    LedStripConfigurationContextType[0]
  >({
    selectedStripPart: null,
    hoveredStripPart: null,
  });

  const ledStripConfigurationContextValue: LedStripConfigurationContextType = [
    ledStripConfiguration,
    {
      setSelectedStripPart: (v) => {
        setLedStripConfiguration({
          selectedStripPart: v,
        });
      },
      setHoveredStripPart: (v) => {
        setLedStripConfiguration({
          hoveredStripPart: v,
        });
      },
    },
  ];

  return (
    <div class="space-y-4">
      <WebSocketListener handlers={webSocketHandlers} />

      <div class="flex items-center justify-between">
        <h1 class="text-xl font-bold text-base-content">{t('ledConfig.title')}</h1>
        <div class="stats shadow">
          <div class="stat py-2 px-4">
            <div class="stat-title text-xs">{t('displays.displayCount')}</div>
            <div class="stat-value text-primary text-lg">{displayStore.displays.length}</div>
          </div>
        </div>
      </div>

      <LedStripConfigurationContext.Provider value={ledStripConfigurationContextValue}>
        <div class="space-y-4">
          {/* LED Strip Sorter Panel */}
          <div class="card bg-base-200 shadow-lg">
            <div class="card-body p-3">
              <div class="card-title text-sm mb-2 flex items-center justify-between gap-2">
                <span class="flex-1 min-w-0">{t('ledConfig.stripSorting')}</span>
                <div class="badge badge-info badge-outline text-xs whitespace-nowrap">{t('ledConfig.realtimePreview')}</div>
              </div>
              <LedStripPartsSorter />
              <div class="text-xs text-base-content/50 mt-2">
                💡 {t('ledConfig.sortingTip')}
              </div>
            </div>
          </div>

          {/* Display Configuration Panel - Auto height based on content */}
          <div class="card bg-base-200 shadow-lg">
            <div class="card-body p-3">
              <div class="card-title text-sm mb-2 flex items-center justify-between gap-2">
                <span class="flex-1 min-w-0">{t('ledConfig.displayConfiguration')}</span>
                <div class="badge badge-secondary badge-outline text-xs whitespace-nowrap">{t('ledConfig.visualEditor')}</div>
              </div>
              <div class="mb-3">
                <DisplayListContainer>
                  {displayStore.displays.map((display) => (
                    <DisplayView display={display} />
                  ))}
                </DisplayListContainer>
              </div>
              <div class="text-xs text-base-content/50">
                💡 {t('ledConfig.displayTip')}
              </div>
            </div>
          </div>


        </div>
      </LedStripConfigurationContext.Provider>
    </div>
  );
};
