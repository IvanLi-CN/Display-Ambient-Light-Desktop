import { Component, createSignal, createEffect, Show } from 'solid-js';
import { useLanguage } from '../../i18n/index';
import { VirtualLedStrip } from './virtual-display';
import { LedType } from '../../models/led-strip-config';

type LedStripConfigPanelProps = {
  strip: VirtualLedStrip | null;
  onUpdate: (stripId: string, updates: Partial<VirtualLedStrip>) => void;
  onDelete: (stripId: string) => void;
  onClose: () => void;
};

export const LedStripConfigPanel: Component<LedStripConfigPanelProps> = (props) => {
  const { t } = useLanguage();
  
  // 本地状态用于表单控制
  const [localCount, setLocalCount] = createSignal(0);
  const [localReversed, setLocalReversed] = createSignal(false);
  const [localLedType, setLocalLedType] = createSignal<LedType>(LedType.WS2812B);
  const [localDriverId, setLocalDriverId] = createSignal(1);
  const [localStripOrder, setLocalStripOrder] = createSignal(1);
  const [localStartOffset, setLocalStartOffset] = createSignal(0);
  const [localEndOffset, setLocalEndOffset] = createSignal(100);

  // 当选中的灯带改变时，更新本地状态
  createEffect(() => {
    if (props.strip) {
      setLocalCount(props.strip.count);
      setLocalReversed(props.strip.reversed);
      setLocalLedType(props.strip.ledType);
      setLocalDriverId(props.strip.driverId);
      setLocalStripOrder(props.strip.stripOrder);
      setLocalStartOffset(props.strip.startOffset);
      setLocalEndOffset(props.strip.endOffset);
    }
  });

  // 更新函数
  const updateStrip = (updates: Partial<VirtualLedStrip>) => {
    if (props.strip) {
      props.onUpdate(props.strip.id, updates);
    }
  };

  // 数量控制
  const handleCountChange = (delta: number) => {
    const newCount = Math.max(1, Math.min(1000, localCount() + delta));
    setLocalCount(newCount);
    updateStrip({ count: newCount });
  };

  const handleCountInput = (value: string) => {
    const count = parseInt(value) || 1;
    const clampedCount = Math.max(1, Math.min(1000, count));
    setLocalCount(clampedCount);
    updateStrip({ count: clampedCount });
  };

  // 反向控制
  const handleReversedToggle = () => {
    const newReversed = !localReversed();
    setLocalReversed(newReversed);
    updateStrip({ reversed: newReversed });
  };

  // LED类型控制
  const handleLedTypeChange = (type: LedType) => {
    setLocalLedType(type);
    updateStrip({ ledType: type });
  };

  // 驱动器控制
  const handleDriverIdChange = (driverId: number) => {
    setLocalDriverId(driverId);
    updateStrip({ driverId });
    // TODO: 自动更新序号为该驱动器的最大序号+1
  };

  // 序号控制
  const handleStripOrderChange = (order: number) => {
    const clampedOrder = Math.max(1, order);
    setLocalStripOrder(clampedOrder);
    updateStrip({ stripOrder: clampedOrder });
  };

  // 偏移控制
  const handleStartOffsetChange = (offset: number) => {
    const clampedOffset = Math.max(0, Math.min(100, offset));
    setLocalStartOffset(clampedOffset);
    updateStrip({ startOffset: clampedOffset });
  };

  const handleEndOffsetChange = (offset: number) => {
    const clampedOffset = Math.max(0, Math.min(100, offset));
    setLocalEndOffset(clampedOffset);
    updateStrip({ endOffset: clampedOffset });
  };

  // 删除灯带
  const handleDelete = () => {
    if (props.strip && confirm(t('singleDisplayConfig.confirmDeleteStrip'))) {
      props.onDelete(props.strip.id);
    }
  };

  return (
    <Show when={props.strip}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-base-100 rounded-lg shadow-xl w-96 max-h-[80vh] overflow-y-auto">
          {/* 标题栏 */}
          <div class="flex items-center justify-between p-4 border-b border-base-300">
            <h3 class="text-lg font-semibold">
              {t('singleDisplayConfig.stripConfig')} - {t(`ledConfig.${props.strip!.border.toLowerCase()}`)}
            </h3>
            <button 
              class="btn btn-ghost btn-sm btn-circle"
              onClick={props.onClose}
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>

          {/* 配置表单 */}
          <div class="p-4 space-y-4">
            {/* LED数量 */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{t('ledConfig.ledCount')}</span>
              </label>
              <div class="flex items-center gap-2">
                <button 
                  class="btn btn-outline btn-sm"
                  onClick={() => handleCountChange(-1)}
                >
                  -
                </button>
                <input
                  type="number"
                  class="input input-bordered flex-1 text-center"
                  value={localCount()}
                  min="1"
                  max="1000"
                  onInput={(e) => handleCountInput(e.currentTarget.value)}
                />
                <button 
                  class="btn btn-outline btn-sm"
                  onClick={() => handleCountChange(1)}
                >
                  +
                </button>
              </div>
            </div>

            {/* 数据方向 */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{t('singleDisplayConfig.dataDirection')}</span>
              </label>
              <div class="flex items-center gap-2">
                <span class="text-sm">{t('singleDisplayConfig.normal')}</span>
                <input
                  type="checkbox"
                  class="toggle toggle-primary"
                  checked={localReversed()}
                  onChange={handleReversedToggle}
                />
                <span class="text-sm">{t('singleDisplayConfig.reversed')}</span>
              </div>
            </div>

            {/* LED类型 */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{t('ledConfig.ledType')}</span>
              </label>
              <select 
                class="select select-bordered"
                value={localLedType()}
                onChange={(e) => handleLedTypeChange(e.currentTarget.value as LedType)}
              >
                <option value={LedType.WS2812B}>WS2812B (RGB)</option>
                <option value={LedType.SK6812}>SK6812 (RGBW)</option>
              </select>
            </div>

            {/* 驱动器选择 */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{t('singleDisplayConfig.driverSelection')}</span>
              </label>
              <select 
                class="select select-bordered"
                value={localDriverId()}
                onChange={(e) => handleDriverIdChange(parseInt(e.currentTarget.value))}
              >
                <option value={1}>{t('singleDisplayConfig.driver')} 1</option>
                <option value={2}>{t('singleDisplayConfig.driver')} 2</option>
                <option value={3}>{t('singleDisplayConfig.driver')} 3</option>
                <option value={4}>{t('singleDisplayConfig.driver')} 4</option>
              </select>
            </div>

            {/* 灯带序号 */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{t('singleDisplayConfig.stripOrder')}</span>
              </label>
              <input
                type="number"
                class="input input-bordered"
                value={localStripOrder()}
                min="1"
                onInput={(e) => handleStripOrderChange(parseInt(e.currentTarget.value) || 1)}
              />
            </div>

            {/* 屏幕位置偏移 */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{t('singleDisplayConfig.positionOffset')}</span>
              </label>
              <div class="space-y-2">
                <div>
                  <label class="label-text text-sm">{t('singleDisplayConfig.startOffset')}: {localStartOffset()}%</label>
                  <input
                    type="range"
                    class="range range-primary range-sm"
                    min="0"
                    max="100"
                    value={localStartOffset()}
                    onInput={(e) => handleStartOffsetChange(parseInt(e.currentTarget.value))}
                  />
                </div>
                <div>
                  <label class="label-text text-sm">{t('singleDisplayConfig.endOffset')}: {localEndOffset()}%</label>
                  <input
                    type="range"
                    class="range range-primary range-sm"
                    min="0"
                    max="100"
                    value={localEndOffset()}
                    onInput={(e) => handleEndOffsetChange(parseInt(e.currentTarget.value))}
                  />
                </div>
              </div>
            </div>
          </div>

          {/* 操作按钮 */}
          <div class="flex items-center justify-between p-4 border-t border-base-300">
            <button 
              class="btn btn-error btn-outline btn-sm"
              onClick={handleDelete}
            >
              {t('common.delete')}
            </button>
            <button 
              class="btn btn-primary btn-sm"
              onClick={props.onClose}
            >
              {t('common.close')}
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
