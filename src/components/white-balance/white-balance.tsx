import { listen } from '@tauri-apps/api/event';
import { Component, createEffect, onCleanup } from 'solid-js';
import { ColorCalibration, LedStripConfigContainer } from '../../models/led-strip-config';
import { ledStripStore, setLedStripStore } from '../../stores/led-strip.store';
import { ColorSlider } from './color-slider';
import { TestColorsBg } from './test-colors-bg';
import { invoke } from '@tauri-apps/api/core';
import { VsClose } from 'solid-icons/vs';
import { BiRegularReset } from 'solid-icons/bi';
import transparentBg from '../../assets/transparent-grid-background.svg?url';

const Value: Component<{ value: number }> = (props) => {
  return (
    <div class="badge badge-outline badge-sm font-mono">
      {(props.value * 100).toFixed(0)}%
    </div>
  );
};

export const WhiteBalance = () => {
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

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  const updateColorCalibration = (field: keyof ColorCalibration, value: number) => {
    const calibration = { ...ledStripStore.colorCalibration, [field]: value };
    invoke('set_color_calibration', {
      calibration,
    }).catch((error) => console.log(error));
  };

  const exit = () => {
    window.history.back();
  };

  const reset = () => {
    invoke('set_color_calibration', {
      calibration: new ColorCalibration(),
    }).catch((error) => console.log(error));
  };

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-base-content">白平衡调节</h1>
        <div class="flex gap-2">
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
                  <span class="label-text font-semibold text-base-content/70">白色 (W)</span>
                  <div class="badge badge-outline badge-sm">暂未启用</div>
                </label>
                <ColorSlider class="from-yellow-50 to-cyan-50" disabled />
              </div>
            </div>

            <div class="text-xs text-base-content/50 mt-4">
              💡 提示：调节RGB滑块来校正LED灯条的白平衡，使白色更加纯净
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
