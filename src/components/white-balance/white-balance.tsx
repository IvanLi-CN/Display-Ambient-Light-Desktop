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

const Value: Component<{ value: number }> = (props) => {
  return (
    <div class="badge badge-outline badge-sm font-mono">
      {(props.value * 100).toFixed(0)}%
    </div>
  );
};

export const WhiteBalance = () => {
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
        console.error('Failed to auto enter fullscreen:', error);
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
    setIsDragging(true);
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    setDragOffset({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    });
    e.preventDefault();
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

  // listen to config_changed event
  createEffect(() => {
    const unlisten = listen('config_changed', (event) => {
      const { strips, mappers, color_calibration } =
        event.payload as LedStripConfigContainer;
      console.log(event.payload);
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
    invoke('set_color_calibration', { calibration }).catch((error) =>
      console.log(error),
    );
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
      console.error('Failed to toggle fullscreen:', error);
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
    }).catch((error) => console.log(error));
  };

  return (
    <>
      {/* 普通模式 */}
      {!isFullscreen() && (
        <div class="space-y-6">
          <div class="flex items-center justify-between">
            <h1 class="text-2xl font-bold text-base-content">白平衡调节</h1>
            <div class="flex gap-2">
              <button class="btn btn-outline btn-sm" onClick={toggleFullscreen} title="进入全屏">
                <BsFullscreen size={16} />
                全屏
              </button>
              <button class="btn btn-outline btn-sm" onClick={reset} title="重置到100%">
                <BiRegularReset size={16} />
                重置
              </button>
              <button class="btn btn-primary btn-sm" onClick={exit} title="返回">
                <VsClose size={16} />
                返回
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* 颜色测试区域 */}
            <div class="card bg-base-200 shadow-lg">
              <div class="card-body p-4">
                <div class="card-title text-base mb-3">
                  <span>颜色测试</span>
                  <div class="badge badge-info badge-outline">点击测试</div>
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
                  💡 提示：点击颜色块进行单色测试，再次点击返回多色模式
                </div>
              </div>
            </div>

            {/* 白平衡控制面板 */}
            <div class="card bg-base-200 shadow-lg">
              <div class="card-body p-4">
                <div class="card-title text-base mb-3">
                  <span>RGB调节</span>
                  <div class="badge badge-secondary badge-outline">实时调节</div>
                </div>

                <div class="space-y-4">
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-semibold text-red-500">红色 (R)</span>
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
                      <span class="label-text font-semibold text-green-500">绿色 (G)</span>
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
                      <span class="label-text font-semibold text-blue-500">蓝色 (B)</span>
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
                      <span class="label-text font-semibold text-amber-500">白色 (W)</span>
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
                    💡 白平衡调节使用说明
                  </div>
                  <div class="collapse-content text-xs text-base-content/70 space-y-3">
                    <div class="space-y-2">
                      <p class="font-semibold text-primary">🎯 推荐使用方法：</p>
                      <ol class="list-decimal list-inside space-y-1 ml-2">
                        <li>点击上方"全屏"按钮进入全屏模式</li>
                        <li>全屏模式下屏幕边缘会显示彩色条带</li>
                        <li>将RGB控制面板拖拽到合适位置</li>
                        <li>对比LED灯条颜色与屏幕边缘颜色</li>
                      </ol>
                    </div>

                    <div class="space-y-2">
                      <p class="font-semibold text-secondary">🔧 调节技巧：</p>
                      <ul class="list-disc list-inside space-y-1 ml-2">
                        <li><span class="text-red-500 font-medium">红色偏强</span>：降低R值，LED会减少红色成分</li>
                        <li><span class="text-green-500 font-medium">绿色偏强</span>：降低G值，LED会减少绿色成分</li>
                        <li><span class="text-blue-500 font-medium">蓝色偏强</span>：降低B值，LED会减少蓝色成分</li>
                        <li><span class="text-base-content font-medium">白色发黄</span>：适当提高B值，降低R/G值</li>
                        <li><span class="text-base-content font-medium">白色发蓝</span>：适当降低B值，提高R/G值</li>
                      </ul>
                    </div>

                    <div class="space-y-2">
                      <p class="font-semibold text-accent">📋 对比方法：</p>
                      <ul class="list-disc list-inside space-y-1 ml-2">
                        <li>重点观察白色区域，确保LED白光与屏幕白色一致</li>
                        <li>检查彩色区域，确保LED颜色饱和度合适</li>
                        <li>在不同环境光下测试，确保效果稳定</li>
                        <li>调节完成后可点击"重置"按钮恢复默认值</li>
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
            class="fixed w-80 bg-base-200/95 backdrop-blur-sm rounded-lg shadow-xl z-60 cursor-move select-none"
            style={{
              left: `${panelPosition().x}px`,
              top: `${panelPosition().y}px`,
              transform: 'none'
            }}
            onMouseDown={handleMouseDown}
          >
            <div class="card-body p-4">
              <div class="card-title text-base mb-3 flex justify-between items-center">
                <div class="flex items-center gap-2">
                  <span class="text-xs opacity-60">⋮⋮</span>
                  <span>RGB调节</span>
                  <div class="badge badge-secondary badge-outline">可拖拽</div>
                </div>
                <button class="btn btn-ghost btn-xs" onClick={toggleFullscreen} title="退出全屏">
                  <BsFullscreenExit size={14} />
                </button>
              </div>

              <div class="space-y-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold text-red-500">红色 (R)</span>
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
                    <span class="label-text font-semibold text-green-500">绿色 (G)</span>
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
                    <span class="label-text font-semibold text-blue-500">蓝色 (B)</span>
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
                    <span class="label-text font-semibold text-amber-500">白色 (W)</span>
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
                    <span class="label-text font-semibold text-base-content/70">白色 (W)</span>
                    <div class="badge badge-outline badge-sm">暂未启用</div>
                  </label>
                  <ColorSlider class="from-yellow-50 to-cyan-50" disabled />
                </div>
              </div>

              <div class="text-xs text-base-content/60 mt-3 p-2 bg-base-300/50 rounded">
                💡 对比屏幕边缘颜色与LED灯条，调节RGB滑块使颜色一致
              </div>

              <div class="flex gap-2 mt-4">
                <button class="btn btn-outline btn-sm flex-1" onClick={reset} title="重置到100%">
                  <BiRegularReset size={14} />
                  重置
                </button>
                <button class="btn btn-primary btn-sm flex-1" onClick={exit} title="返回">
                  <VsClose size={14} />
                  返回
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
};