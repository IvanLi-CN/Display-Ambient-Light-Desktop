import { listen } from '@tauri-apps/api/event';
import { Component, createEffect, onCleanup, createSignal } from 'solid-js';
import { ColorCalibration, LedStripConfigContainer } from '../../models/led-strip-config';
import { ledStripStore, setLedStripStore } from '../../stores/led-strip.store';
import { ColorSlider } from './color-slider';
import { TestColorsBg } from './test-colors-bg';
import { invoke } from '@tauri-apps/api/core';
import { VsClose } from 'solid-icons/vs';
import { BiRegularReset } from 'solid-icons/bi';
import { BsFullscreen, BsFullscreenExit } from 'solid-icons/bs';
import { getCurrentWindow } from '@tauri-apps/api/window';
import transparentBg from '../../assets/transparent-grid-background.svg?url';
import { useLanguage } from '../../i18n/index';

const Value: Component<{ value: number }> = (props) => {
  return (
    <div class="badge badge-outline badge-sm font-mono">
      {(props.value * 100).toFixed(0)}%
    </div>
  );
};

export const WhiteBalance = () => {
  const { t } = useLanguage();
  const [isFullscreen, setIsFullscreen] = createSignal(false);
  const [panelPosition, setPanelPosition] = createSignal({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = createSignal(false);
  const [dragOffset, setDragOffset] = createSignal({ x: 0, y: 0 });

  // 自动进入全屏模式
  createEffect(() => {
    const autoEnterFullscreen = async () => {
      try {
        const window = getCurrentWindow();
        const currentFullscreen = await window.isFullscreen();
        if (!currentFullscreen) {
          await window.setFullscreen(true);
          setIsFullscreen(true);
        } else {
          setIsFullscreen(true);
        }
      } catch (error) {
        // Silently handle fullscreen error
      }
    };

    autoEnterFullscreen();
  });

  // 初始化面板位置到屏幕中央
  createEffect(() => {
    if (isFullscreen()) {
      const centerX = window.innerWidth / 2 - 160; // 160是面板宽度的一半
      const centerY = window.innerHeight / 2 - 200; // 200是面板高度的一半
      setPanelPosition({ x: centerX, y: centerY });
    }
  });

  // 拖拽处理函数
  const handleMouseDown = (e: MouseEvent) => {
    // 确保只有在标题栏区域点击时才触发拖拽
    setIsDragging(true);
    const panelRect = (e.currentTarget as HTMLElement).closest('.fixed')?.getBoundingClientRect();
    if (panelRect) {
      setDragOffset({
        x: e.clientX - panelRect.left,
        y: e.clientY - panelRect.top
      });
    }
    e.preventDefault();
    e.stopPropagation();
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (isDragging()) {
      const newX = e.clientX - dragOffset().x;
      const newY = e.clientY - dragOffset().y;

      // 限制面板在屏幕范围内
      const maxX = window.innerWidth - 320; // 320是面板宽度
      const maxY = window.innerHeight - 400; // 400是面板高度

      setPanelPosition({
        x: Math.max(0, Math.min(newX, maxX)),
        y: Math.max(0, Math.min(newY, maxY))
      });
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  // 添加全局鼠标事件监听
  createEffect(() => {
    if (isDragging()) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    } else {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    }
  });

  // 监听ESC键和窗口全屏状态变化
  createEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isFullscreen()) {
        toggleFullscreen();
      }
    };

    const checkFullscreenStatus = async () => {
      try {
        const window = getCurrentWindow();
        const currentFullscreen = await window.isFullscreen();
        if (currentFullscreen !== isFullscreen()) {
          setIsFullscreen(currentFullscreen);
          // 退出全屏时重置面板位置
          if (!currentFullscreen) {
            setPanelPosition({ x: 0, y: 0 });
          }
        }
      } catch (error) {
        // Silently handle error
      }
    };

    // 定期检查全屏状态
    const intervalId = setInterval(checkFullscreenStatus, 100);

    document.addEventListener('keydown', handleKeyDown);

    onCleanup(() => {
      document.removeEventListener('keydown', handleKeyDown);
      clearInterval(intervalId);
    });
  });

  // listen to config_changed event
  createEffect(() => {
    const unlisten = listen('config_changed', (event) => {
      const { strips, mappers, color_calibration } =
        event.payload as LedStripConfigContainer;
      setLedStripStore({
        strips,
        mappers,
        colorCalibration: color_calibration,
      });
    });

    onCleanup(async () => {
      (await unlisten)();
    });
  });

  const updateColorCalibration = (
    key: keyof ColorCalibration,
    value: number,
  ) => {
    const calibration = { ...ledStripStore.colorCalibration };
    calibration[key] = value;
    setLedStripStore('colorCalibration', calibration);
    invoke('set_color_calibration', { calibration }).catch(() => {
      // Silently handle error
    });
  };

  const toggleFullscreen = async () => {
    try {
      const window = getCurrentWindow();
      const currentFullscreen = await window.isFullscreen();
      await window.setFullscreen(!currentFullscreen);
      setIsFullscreen(!currentFullscreen);

      // 退出全屏时重置面板位置
      if (currentFullscreen) {
        setPanelPosition({ x: 0, y: 0 });
      }
    } catch (error) {
      // Silently handle fullscreen error
    }
  };

  const exit = () => {
    // 退出时确保退出全屏模式
    if (isFullscreen()) {
      toggleFullscreen().then(() => {
        window.history.back();
      });
    } else {
      window.history.back();
    }
  };

  const reset = () => {
    invoke('set_color_calibration', {
      calibration: new ColorCalibration(),
    }).catch(() => {
      // Silently handle error
    });
  };

  return (
    <>
      {/* 普通模式 */}
      {!isFullscreen() && (
        <div class="space-y-6">
          <div class="flex items-center justify-between">
            <h1 class="text-2xl font-bold text-base-content">{t('whiteBalance.title')}</h1>
            <div class="flex gap-2">
              <button class="btn btn-outline btn-sm" onClick={toggleFullscreen} title={t('common.fullscreen')}>
                <BsFullscreen size={16} />
                {t('common.fullscreen')}
              </button>
              <button class="btn btn-outline btn-sm" onClick={reset} title={t('common.reset')}>
                <BiRegularReset size={16} />
                {t('common.reset')}
              </button>
              <button class="btn btn-primary btn-sm" onClick={exit} title={t('whiteBalance.back')}>
                <VsClose size={16} />
                {t('whiteBalance.back')}
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* 颜色测试区域 */}
            <div class="card bg-base-200 shadow-lg">
              <div class="card-body p-4">
                <div class="card-title text-base mb-3">
                  <span>{t('whiteBalance.colorTest')}</span>
                  <div class="badge badge-info badge-outline">{t('whiteBalance.clickToTest')}</div>
                </div>
                <div
                  class="aspect-square rounded-lg overflow-hidden border border-base-300"
                  style={{
                    'background-image': `url(${transparentBg})`,
                  }}
                >
                  <TestColorsBg />
                </div>
                <div class="text-xs text-base-content/50 mt-2">
                  💡 {t('whiteBalance.colorTestTip')}
                </div>
              </div>
            </div>

            {/* 白平衡控制面板 */}
            <div class="card bg-base-200 shadow-lg">
              <div class="card-body p-4">
                <div class="card-title text-base mb-3">
                  <span>{t('whiteBalance.rgbAdjustment')}</span>
                  <div class="badge badge-secondary badge-outline">{t('whiteBalance.realtimeAdjustment')}</div>
                </div>

                <div class="space-y-4">
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-red-500">{t('whiteBalance.redChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.r} />
                    </label>
                    <ColorSlider
                      class="from-cyan-500 to-red-500"
                      value={ledStripStore.colorCalibration.r}
                      onInput={(ev) =>
                        updateColorCalibration(
                          'r',
                          (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                        )
                      }
                    />
                  </div>

                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-green-500">{t('whiteBalance.greenChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.g} />
                    </label>
                    <ColorSlider
                      class="from-pink-500 to-green-500"
                      value={ledStripStore.colorCalibration.g}
                      onInput={(ev) =>
                        updateColorCalibration(
                          'g',
                          (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                        )
                      }
                    />
                  </div>

                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-blue-500">{t('whiteBalance.blueChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.b} />
                    </label>
                    <ColorSlider
                      class="from-yellow-500 to-blue-500"
                      value={ledStripStore.colorCalibration.b}
                      onInput={(ev) =>
                        updateColorCalibration(
                          'b',
                          (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                        )
                      }
                    />
                  </div>

                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-amber-500">{t('whiteBalance.whiteChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.w} />
                    </label>
                    <ColorSlider
                      class="from-amber-100 to-amber-50"
                      value={ledStripStore.colorCalibration.w}
                      onInput={(ev) =>
                        updateColorCalibration(
                          'w',
                          (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                        )
                      }
                    />
                  </div>
                </div>

                {/* 使用说明 - 可展开 */}
                <div class="collapse collapse-arrow bg-base-100 mt-4">
                  <input type="checkbox" />
                  <div class="collapse-title text-sm font-medium text-base-content/80">
                    💡 {t('whiteBalance.usageInstructions')}
                  </div>
                  <div class="collapse-content text-xs text-base-content/70 space-y-3">
                    <div class="space-y-2">
                      <p class="font-semibold text-primary">{t('whiteBalance.recommendedMethod')}</p>
                      <ol class="list-decimal list-inside space-y-1 ml-2">
                        <li>{t('whiteBalance.fullscreenTip')}</li>
                        <li>{t('whiteBalance.dragTip')}</li>
                        <li>{t('whiteBalance.dragPanelTip')}</li>
                        <li>{t('whiteBalance.compareColorsTip')}</li>
                      </ol>
                    </div>

                    <div class="space-y-2">
                      <p class="font-semibold text-secondary">{t('whiteBalance.adjustmentTips')}</p>
                      <ul class="list-disc list-inside space-y-1 ml-2">
                        <li>{t('whiteBalance.redStrong')}</li>
                        <li>{t('whiteBalance.greenStrong')}</li>
                        <li>{t('whiteBalance.blueStrong')}</li>
                        <li>{t('whiteBalance.whiteYellow')}</li>
                        <li>{t('whiteBalance.whiteBlue')}</li>
                      </ul>
                    </div>

                    <div class="space-y-2">
                      <p class="font-semibold text-accent">{t('whiteBalance.comparisonMethod')}</p>
                      <ul class="list-disc list-inside space-y-1 ml-2">
                        <li>{t('whiteBalance.whiteComparison')}</li>
                        <li>{t('whiteBalance.colorComparison')}</li>
                        <li>{t('whiteBalance.environmentTest')}</li>
                        <li>{t('whiteBalance.resetNote')}</li>
                      </ul>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 全屏模式 */}
      {isFullscreen() && (
        <div class="fixed inset-0 w-screen h-screen bg-black z-50">
          {/* 全屏颜色测试区域 - 紧贴边缘 */}
          <div class="absolute inset-0 w-full h-full">
            <TestColorsBg />
          </div>

          {/* 可拖拽的RGB控制面板 */}
          <div
            class="fixed w-80 bg-base-200/95 backdrop-blur-sm rounded-lg shadow-xl z-60 select-none"
            style={{
              left: `${panelPosition().x}px`,
              top: `${panelPosition().y}px`,
              transform: 'none'
            }}
          >
            <div class="card-body p-4">
              <div
                class="card-title text-base mb-3 flex justify-between items-center cursor-move"
                onMouseDown={handleMouseDown}
              >
                <div class="flex items-center gap-2">
                  <span class="text-xs opacity-60">⋮⋮</span>
                  <span>{t('whiteBalance.rgbAdjustment')}</span>
                  <div class="badge badge-secondary badge-outline">{t('whiteBalance.draggable')}</div>
                </div>
                <button class="btn btn-ghost btn-xs cursor-pointer" onClick={toggleFullscreen} title={t('whiteBalance.exitFullscreen')}>
                  <BsFullscreenExit size={14} />
                </button>
              </div>

              <div class="space-y-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-red-500">{t('whiteBalance.redChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.r} />
                  </label>
                  <ColorSlider
                    class="from-cyan-500 to-red-500"
                    value={ledStripStore.colorCalibration.r}
                    onInput={(ev) =>
                      updateColorCalibration(
                        'r',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-green-500">{t('whiteBalance.greenChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.g} />
                  </label>
                  <ColorSlider
                    class="from-pink-500 to-green-500"
                    value={ledStripStore.colorCalibration.g}
                    onInput={(ev) =>
                      updateColorCalibration(
                        'g',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-blue-500">{t('whiteBalance.blueChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.b} />
                  </label>
                  <ColorSlider
                    class="from-yellow-500 to-blue-500"
                    value={ledStripStore.colorCalibration.b}
                    onInput={(ev) =>
                      updateColorCalibration(
                        'b',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-amber-500">{t('whiteBalance.whiteChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.w} />
                  </label>
                  <ColorSlider
                    class="from-amber-100 to-amber-50"
                    value={ledStripStore.colorCalibration.w}
                    onInput={(ev) =>
                      updateColorCalibration(
                        'w',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-base-content/70">{t('whiteBalance.whiteChannel')}</span>
                    <div class="badge badge-outline badge-sm">{t('whiteBalance.notEnabled')}</div>
                  </label>
                  <ColorSlider class="from-yellow-50 to-cyan-50 opacity-50 pointer-events-none" />
                </div>
              </div>

              <div class="text-xs text-base-content/60 mt-3 p-2 bg-base-300/50 rounded">
                💡 {t('whiteBalance.fullscreenComparisonTip')}
              </div>

              <div class="flex gap-2 mt-4">
                <button class="btn btn-outline btn-sm flex-1" onClick={reset} title={t('common.reset')}>
                  <BiRegularReset size={14} />
                  {t('common.reset')}
                </button>
                <button class="btn btn-primary btn-sm flex-1" onClick={exit} title={t('whiteBalance.back')}>
                  <VsClose size={14} />
                  {t('whiteBalance.back')}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
};