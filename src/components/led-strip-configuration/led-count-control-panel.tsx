import { invoke } from '@tauri-apps/api/core';
import { Component, createMemo, For, JSX, splitProps, useContext } from 'solid-js';
import { DisplayInfo } from '../../models/display-info.model';
import { ledStripStore } from '../../stores/led-strip.store';
import { Borders } from '../../constants/border';
import { LedType } from '../../models/led-strip-config';
import { LedStripConfigurationContext } from '../../contexts/led-strip-configuration.context';
import { useLanguage } from '../../i18n/index';

type LedCountControlItemProps = {
  displayId: number;
  border: Borders;
  label: string;
};

const LedCountControlItem: Component<LedCountControlItemProps> = (props) => {
  const [stripConfiguration, { setHoveredStripPart }] = useContext(LedStripConfigurationContext);
  const { t } = useLanguage();

  const config = createMemo(() => {
    return ledStripStore.strips.find(
      (s) => s.display_id === props.displayId && s.border === props.border
    );
  });

  const handleDecrease = () => {
    if (config()) {
      invoke('patch_led_strip_len', {
        displayId: props.displayId,
        border: props.border,
        deltaLen: -1,
      }).catch((e) => {
        console.error(e);
      });
    }
  };

  const handleIncrease = () => {
    if (config()) {
      invoke('patch_led_strip_len', {
        displayId: props.displayId,
        border: props.border,
        deltaLen: 1,
      }).catch((e) => {
        console.error(e);
      });
    }
  };

  const handleInputChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const newValue = parseInt(target.value);
    const currentLen = config()?.len || 0;

    if (!isNaN(newValue) && newValue >= 0 && newValue <= 1000) {
      const deltaLen = newValue - currentLen;
      if (deltaLen !== 0) {
        invoke('patch_led_strip_len', {
          displayId: props.displayId,
          border: props.border,
          deltaLen: deltaLen,
        }).catch((e) => {
          console.error(e);
          // Reset input value on error
          target.value = currentLen.toString();
        });
      }
    } else {
      // Reset invalid input
      target.value = (config()?.len || 0).toString();
    }
  };

  const handleLedTypeChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const newType = target.value as LedType;

    invoke('patch_led_strip_type', {
      displayId: props.displayId,
      border: props.border,
      ledType: newType,
    }).catch((e) => {
      console.error(e);
    });
  };

  const onMouseEnter = () => {
    setHoveredStripPart({
      displayId: props.displayId,
      border: props.border,
    });
  };

  const onMouseLeave = () => {
    setHoveredStripPart(null);
  };

  return (
    <div
      class="card bg-base-100 border border-base-300/50 p-1.5 transition-all duration-200 cursor-pointer"
      classList={{
        'ring-2 ring-primary bg-primary/20 border-primary':
          stripConfiguration.hoveredStripPart?.border === props.border &&
          stripConfiguration.hoveredStripPart?.displayId === props.displayId,
      }}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
    >
      <div class="flex flex-col gap-1">
        <div class="text-center">
          <span class="text-xs font-medium text-base-content">
            {props.label}
          </span>
        </div>

        <div class="flex items-center gap-0.5">
          <button
            class="btn btn-xs btn-circle btn-outline flex-shrink-0 min-h-0 h-6 w-6"
            onClick={handleDecrease}
            disabled={!config() || (config()?.len || 0) <= 0}
            title={t('ledConfig.decreaseLedCount')}
          >
            -
          </button>

          <input
            type="number"
            class="input input-xs flex-1 text-center min-w-0 text-xs font-medium [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none h-6 px-1"
            value={config()?.len || 0}
            min="0"
            max="1000"
            onBlur={handleInputChange}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleInputChange(e);
              }
            }}
          />

          <button
            class="btn btn-xs btn-circle btn-outline flex-shrink-0 min-h-0 h-6 w-6"
            onClick={handleIncrease}
            disabled={!config() || (config()?.len || 0) >= 1000}
            title={t('ledConfig.increaseLedCount')}
          >
            +
          </button>
        </div>

        <div class="mt-1">
          <select
            class="select select-xs w-full text-xs h-6 min-h-0"
            value={config()?.led_type || LedType.WS2812B}
            onChange={handleLedTypeChange}
            title={t('ledConfig.ledType')}
          >
            <option value={LedType.WS2812B}>WS2812B</option>
            <option value={LedType.SK6812}>SK6812</option>
          </select>
        </div>
      </div>
    </div>
  );
};

type LedCountControlPanelProps = {
  display: DisplayInfo;
} & JSX.HTMLAttributes<HTMLDivElement>;

export const LedCountControlPanel: Component<LedCountControlPanelProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['display']);
  const { t } = useLanguage();

  const borders: { border: Borders; label: string }[] = [
    { border: 'Top', label: t('ledConfig.top') },
    { border: 'Bottom', label: t('ledConfig.bottom') },
    { border: 'Left', label: t('ledConfig.left') },
    { border: 'Right', label: t('ledConfig.right') },
  ];

  return (
    <div {...rootProps} class={'card bg-base-200 shadow-lg border border-base-300 ' + (rootProps.class || '')}>
      <div class="card-body p-3">
        <div class="card-title text-sm mb-2 flex items-center justify-between">
          <span>{t('ledConfig.ledCountControl')}</span>
          <div class="badge badge-info badge-outline text-xs">{t('ledConfig.display')} {localProps.display.id}</div>
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
          <For each={borders}>
            {(item) => (
              <LedCountControlItem
                displayId={localProps.display.id}
                border={item.border}
                label={item.label}
              />
            )}
          </For>
        </div>

        <div class="text-xs text-base-content/50 mt-2 p-1.5 bg-base-300/50 rounded">
          💡 {t('ledConfig.controlTip')}
        </div>
      </div>
    </div>
  );
};
