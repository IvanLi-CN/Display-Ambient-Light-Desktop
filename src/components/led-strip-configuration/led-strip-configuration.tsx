import { createEffect, createMemo, For } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import { WebSocketListener } from '../websocket-listener';
import { DisplayView } from './display-view';
import { DisplayListContainer } from './display-list-container';
import { displayStore, setDisplayStore } from '../../stores/display.store';
import { LedStripConfigContainer } from '../../models/led-strip-config';
import { setLedStripStore } from '../../stores/led-strip.store';
import { LedStripPartsSorter } from './led-strip-parts-sorter';
import { LedCountControlPanel } from './led-count-control-panel';
import { createStore } from 'solid-js/store';
import {
  LedStripConfigurationContext,
  LedStripConfigurationContextType,
} from '../../contexts/led-strip-configuration.context';
import { useLanguage } from '../../i18n/index';


export const LedStripConfiguration = () => {
  const { t } = useLanguage();

  console.log('🔧 LedStripConfiguration component loaded');

  createEffect(() => {
    adaptiveApi.listDisplayInfo().then((displays) => {
      const parsedDisplays = JSON.parse(displays);
      setDisplayStore({
        displays: parsedDisplays,
      });
    }).catch((error) => {
      console.error('Failed to load displays:', error);
    });

    adaptiveApi.getConfig().then((configs) => {
      setLedStripStore(configs);
    }).catch((error) => {
      console.error('Failed to load LED strip configs:', error);
    });
  });

  // WebSocket event handlers
  const webSocketHandlers = {
    onConfigChanged: (data: any) => {
      console.log('🔧 配置变化事件:', data);
      try {
        const configData = data as LedStripConfigContainer;
        // 安全检查：确保 strips 存在且是数组
        if (configData && configData.strips && Array.isArray(configData.strips)) {
          setLedStripStore({
            strips: configData.strips,
          });
        } else {
          console.warn('⚠️ 配置数据无效或缺少strips:', configData);
        }
      } catch (error) {
        console.error('❌ 处理配置变化事件失败:', error);
      }
    },
    onLedColorsChanged: (data: any) => {
      if (!window.document.hidden) {
        console.log('🎨 LED颜色变化事件:', data);
        // 数据应该是 { colors: Vec<u8> } 格式
        const colors = data.colors || data;
        const colorsArray = new Uint8ClampedArray(colors);
        setLedStripStore({
          colors: colorsArray,
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

          {/* LED Count Control Panels */}
          <div class="flex-shrink-0">
            <div class="flex items-center gap-2 mb-2">
              <h2 class="text-base font-semibold text-base-content">{t('ledConfig.ledCountControl')}</h2>
              <div class="badge badge-info badge-outline text-xs">{t('ledConfig.realtimeAdjustment')}</div>
            </div>
            <div class="led-control-grid">
              {displayStore.displays.map((display) => (
                <LedCountControlPanel display={display} />
              ))}
            </div>
          </div>
        </div>
      </LedStripConfigurationContext.Provider>
    </div>
  );
};
