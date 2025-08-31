import { Component, createEffect, onCleanup, createSignal } from 'solid-js';
import { ColorCalibration, LedStripConfigContainer } from '../../models/led-strip-config';
import { ledStripStore, setLedStripStore } from '../../stores/led-strip.store';
import { ColorSlider } from './color-slider';
import { TestColorsBg } from './test-colors-bg';
import { VsClose } from 'solid-icons/vs';
import { BiRegularReset } from 'solid-icons/bi';
import { BsFullscreen, BsFullscreenExit } from 'solid-icons/bs';
import transparentBg from '../../assets/transparent-grid-background.svg?url';
import { useLanguage } from '../../i18n/index';
import { adaptiveApi } from '../../services/api-adapter';
import { colorCalibrationService } from '../../services/color-calibration.service';

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

  // 初始化全屏状态检测
  createEffect(() => {
    const checkInitialFullscreenStatus = () => {
      setIsFullscreen(!!document.fullscreenElement);
    };

    checkInitialFullscreenStatus();
  });

  // 组件卸载时禁用颜色校准模式
  onCleanup(async () => {
    try {
      await colorCalibrationService.disableColorCalibrationMode();
      console.log('🎨 颜色校准模式已禁用');
    } catch (error) {
      console.error('❌ 禁用颜色校准模式失败:', error);
    }
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

    const checkFullscreenStatus = () => {
      const currentFullscreen = !!document.fullscreenElement;
      if (currentFullscreen !== isFullscreen()) {
        setIsFullscreen(currentFullscreen);
        // 退出全屏时重置面板位置
        if (!currentFullscreen) {
          setPanelPosition({ x: 0, y: 0 });
        }
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
    let unlisten: (() => void) | null = null;

    adaptiveApi.onEvent('onConfigChanged', (data: LedStripConfigContainer) => {
      const { strips, color_calibration } = data;
      setLedStripStore({
        strips,
        colorCalibration: color_calibration,
      });
    }).then((unlistenFn) => {
      unlisten = unlistenFn;
    }).catch((error) => {
      console.warn('Failed to listen to config_changed event:', error);
    });

    onCleanup(() => {
      if (unlisten) {
        unlisten();
      }
    });
  });

  const updateColorCalibration = async (
    key: keyof ColorCalibration,
    value: number,
  ) => {
    const calibration = { ...ledStripStore.colorCalibration };
    calibration[key] = value;
    setLedStripStore('colorCalibration', calibration);

    try {
      // 只更新配置，不启用颜色校准模式
      // 颜色校准模式只有在用户点击颜色块进行测试时才启用
      await adaptiveApi.updateGlobalColorCalibration(calibration);
      console.log('✅ 颜色校准配置已更新，但未启用校准模式');
    } catch (error) {
      console.error('❌ 更新颜色校准失败:', error);
    }
  };

  const toggleFullscreen = async () => {
    try {
      if (!document.fullscreenElement) {
        // 进入全屏
        await document.documentElement.requestFullscreen();
        setIsFullscreen(true);
      } else {
        // 退出全屏
        await document.exitFullscreen();
        setIsFullscreen(false);
        // 退出全屏时重置面板位置
        setPanelPosition({ x: 0, y: 0 });
      }
    } catch (error) {
      // Silently handle fullscreen error
      console.warn('全屏操作失败:', error);
    }
  };

  const exit = async () => {
    try {
      // 禁用颜色校准模式
      await colorCalibrationService.disableColorCalibrationMode();

      // 退出时确保退出全屏模式
      if (isFullscreen()) {
        await toggleFullscreen();
      }

      // 返回上一页
      window.history.back();
    } catch (error) {
      console.error('❌ 退出颜色校准界面失败:', error);
      // 即使出错也要返回上一页
      window.history.back();
    }
  };

  const reset = () => {
    adaptiveApi.updateGlobalColorCalibration(new ColorCalibration()).catch(() => {
      // Silently handle error
    });
  };

  return (
    <>
      {/* 普通模式 */}
      {!isFullscreen() && (
        <div class="space-y-6">
          <div class="flex items-center justify-between">
            <h1 class="text-2xl font-bold text-base-content">{t('colorCalibration.title')}</h1>
            <div class="flex gap-2">
              <button class="btn btn-outline btn-sm" onClick={toggleFullscreen} title={t('common.fullscreen')}>
                <BsFullscreen size={16} />
                {t('common.fullscreen')}
              </button>
              <button class="btn btn-outline btn-sm" onClick={reset} title={t('common.reset')}>
                <BiRegularReset size={16} />
                {t('common.reset')}
              </button>
              <button class="btn btn-primary btn-sm" onClick={exit} title={t('colorCalibration.back')}>
                <VsClose size={16} />
                {t('colorCalibration.back')}
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* 颜色测试区域 */}
            <div class="card bg-base-200 shadow-lg">
              <div class="card-body p-4">
                <div class="card-title text-base mb-3 flex items-center justify-between gap-2">
                  <span class="flex-1 min-w-0">{t('colorCalibration.colorTest')}</span>
                  <div class="badge badge-info badge-outline whitespace-nowrap">{t('colorCalibration.clickToTest')}</div>
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
                  💡 {t('colorCalibration.colorTestTip')}
                </div>
              </div>
            </div>

            {/* 白平衡控制面板 */}
            <div class="card bg-base-200 shadow-lg">
              <div class="card-body p-4">
                <div class="card-title text-base mb-3">
                  <span>{t('colorCalibration.rgbAdjustment')}</span>
                  <div class="badge badge-secondary badge-outline">{t('colorCalibration.realtimeAdjustment')}</div>
                </div>

                <div class="space-y-4">
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-red-500">{t('colorCalibration.redChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.r} />
                    </label>
                    <ColorSlider
                      class="from-cyan-500 to-red-500"
                      value={ledStripStore.colorCalibration.r}
                      onInput={async (ev) =>
                        await updateColorCalibration(
                          'r',
                          (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                        )
                      }
                    />
                  </div>

                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-green-500">{t('colorCalibration.greenChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.g} />
                    </label>
                    <ColorSlider
                      class="from-pink-500 to-green-500"
                      value={ledStripStore.colorCalibration.g}
                      onInput={async (ev) =>
                        await updateColorCalibration(
                          'g',
                          (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                        )
                      }
                    />
                  </div>

                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-blue-500">{t('colorCalibration.blueChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.b} />
                    </label>
                    <ColorSlider
                      class="from-yellow-500 to-blue-500"
                      value={ledStripStore.colorCalibration.b}
                      onInput={async (ev) =>
                        await updateColorCalibration(
                          'b',
                          (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                        )
                      }
                    />
                  </div>

                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-amber-500">{t('colorCalibration.whiteChannel')}</span>
                      <Value value={ledStripStore.colorCalibration.w} />
                    </label>
                    <ColorSlider
                      class="from-amber-100 to-amber-50"
                      value={ledStripStore.colorCalibration.w}
                      onInput={async (ev) =>
                        await updateColorCalibration(
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
                    💡 {t('colorCalibration.usageInstructions')}
                  </div>
                  <div class="collapse-content text-xs text-base-content/70 space-y-3">
                    <div class="space-y-2">
                      <p class="font-semibold text-primary">{t('colorCalibration.recommendedMethod')}</p>
                      <ol class="list-decimal list-inside space-y-1 ml-2">
                        <li>{t('colorCalibration.fullscreenTip')}</li>
                        <li>{t('colorCalibration.dragTip')}</li>
                        <li>{t('colorCalibration.dragPanelTip')}</li>
                        <li>{t('colorCalibration.compareColorsTip')}</li>
                      </ol>
                    </div>

                    <div class="space-y-2">
                      <p class="font-semibold text-secondary">{t('colorCalibration.adjustmentTips')}</p>
                      <ul class="list-disc list-inside space-y-1 ml-2">
                        <li>{t('colorCalibration.redStrong')}</li>
                        <li>{t('colorCalibration.greenStrong')}</li>
                        <li>{t('colorCalibration.blueStrong')}</li>
                        <li>{t('colorCalibration.whiteYellow')}</li>
                        <li>{t('colorCalibration.whiteBlue')}</li>
                      </ul>
                    </div>

                    <div class="space-y-2">
                      <p class="font-semibold text-accent">{t('colorCalibration.comparisonMethod')}</p>
                      <ul class="list-disc list-inside space-y-1 ml-2">
                        <li>{t('colorCalibration.whiteComparison')}</li>
                        <li>{t('colorCalibration.colorComparison')}</li>
                        <li>{t('colorCalibration.environmentTest')}</li>
                        <li>{t('colorCalibration.resetNote')}</li>
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
                class="card-title text-base mb-3 flex justify-between items-center cursor-move gap-2"
                onMouseDown={handleMouseDown}
              >
                <div class="flex items-center gap-2 flex-1 min-w-0">
                  <span class="text-xs opacity-60">⋮⋮</span>
                  <span>{t('colorCalibration.rgbAdjustment')}</span>
                  <div class="badge badge-secondary badge-outline whitespace-nowrap">{t('colorCalibration.draggable')}</div>
                </div>
                <button class="btn btn-ghost btn-xs cursor-pointer flex-shrink-0" onClick={toggleFullscreen} title={t('colorCalibration.exitFullscreen')}>
                  <BsFullscreenExit size={14} />
                </button>
              </div>

              <div class="space-y-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-red-500">{t('colorCalibration.redChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.r} />
                  </label>
                  <ColorSlider
                    class="from-cyan-500 to-red-500"
                    value={ledStripStore.colorCalibration.r}
                    onInput={async (ev) =>
                      await updateColorCalibration(
                        'r',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-green-500">{t('colorCalibration.greenChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.g} />
                  </label>
                  <ColorSlider
                    class="from-pink-500 to-green-500"
                    value={ledStripStore.colorCalibration.g}
                    onInput={async (ev) =>
                      await updateColorCalibration(
                        'g',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-blue-500">{t('colorCalibration.blueChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.b} />
                  </label>
                  <ColorSlider
                    class="from-yellow-500 to-blue-500"
                    value={ledStripStore.colorCalibration.b}
                    onInput={async (ev) =>
                      await updateColorCalibration(
                        'b',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-amber-500">{t('colorCalibration.whiteChannel')}</span>
                    <Value value={ledStripStore.colorCalibration.w} />
                  </label>
                  <ColorSlider
                    class="from-amber-100 to-amber-50"
                    value={ledStripStore.colorCalibration.w}
                    onInput={async (ev) =>
                      await updateColorCalibration(
                        'w',
                        (ev.target as HTMLInputElement).valueAsNumber ?? 1,
                      )
                    }
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-base-content/70">{t('colorCalibration.whiteChannel')}</span>
                    <div class="badge badge-outline badge-sm">{t('colorCalibration.notEnabled')}</div>
                  </label>
                  <ColorSlider class="from-yellow-50 to-cyan-50 opacity-50 pointer-events-none" />
                </div>
              </div>

              <div class="text-xs text-base-content/60 mt-3 p-2 bg-base-300/50 rounded">
                💡 {t('colorCalibration.fullscreenComparisonTip')}
              </div>

              <div class="flex gap-2 mt-4">
                <button class="btn btn-outline btn-sm flex-1" onClick={reset} title={t('common.reset')}>
                  <BiRegularReset size={14} />
                  {t('common.reset')}
                </button>
                <button class="btn btn-primary btn-sm flex-1" onClick={exit} title={t('colorCalibration.back')}>
                  <VsClose size={14} />
                  {t('colorCalibration.back')}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
};