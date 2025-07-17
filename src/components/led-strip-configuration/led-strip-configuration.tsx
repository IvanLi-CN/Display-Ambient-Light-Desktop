import { createEffect, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { DisplayView } from './display-view';
import { DisplayListContainer } from './display-list-container';
import { displayStore, setDisplayStore } from '../../stores/display.store';
import { LedStripConfigContainer } from '../../models/led-strip-config';
import { setLedStripStore } from '../../stores/led-strip.store';
import { listen } from '@tauri-apps/api/event';
import { LedStripPartsSorter } from './led-strip-parts-sorter';
import { LedCountControlPanel } from './led-count-control-panel';
import { createStore } from 'solid-js/store';
import {
  LedStripConfigurationContext,
  LedStripConfigurationContextType,
} from '../../contexts/led-strip-configuration.context';
import { useLanguage } from '../../i18n/index';
import { SingleDisplayConfig } from './single-display-config';


export const LedStripConfiguration = () => {
  const { t } = useLanguage();

  // 临时测试：直接渲染SingleDisplayConfig
  // return <SingleDisplayConfig />;

  console.log('🔧 LedStripConfiguration component loaded');
  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      const parsedDisplays = JSON.parse(displays);
      setDisplayStore({
        displays: parsedDisplays,
      });
    }).catch((error) => {
      console.error('Failed to load displays:', error);
    });

    invoke<LedStripConfigContainer>('read_led_strip_configs').then((configs) => {
      setLedStripStore(configs);
    }).catch((error) => {
      console.error('Failed to load LED strip configs:', error);
    });
  });

  // listen to config_changed event
  createEffect(() => {
    const unlisten = listen('config_changed', (event) => {
      const { strips, mappers } = event.payload as LedStripConfigContainer;
      setLedStripStore({
        strips,
        mappers,
      });
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  // listen to led_colors_changed event
  createEffect(() => {
    const unlisten = listen<Uint8ClampedArray>('led_colors_changed', (event) => {
      if (!window.document.hidden) {
        const colors = event.payload;
        setLedStripStore({
          colors,
        });
      }
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  // listen to led_sorted_colors_changed event
  createEffect(() => {
    const unlisten = listen<Uint8ClampedArray>('led_sorted_colors_changed', (event) => {
      if (!window.document.hidden) {
        const sortedColors = event.payload;
        setLedStripStore({
          sortedColors,
        });
      }
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

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
