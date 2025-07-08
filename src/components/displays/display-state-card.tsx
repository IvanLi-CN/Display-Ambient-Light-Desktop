import { Component, ParentComponent } from 'solid-js';
import { DisplayState } from '../../models/display-state.model';
import { useLanguage } from '../../i18n/index';

type DisplayStateCardProps = {
  state: DisplayState;
};

type ItemProps = {
  label: string;
};

const Item: ParentComponent<ItemProps> = (props) => {
  return (
    <div class="flex justify-between items-center py-1">
      <dt class="text-sm font-medium text-base-content/70">{props.label}</dt>
      <dd class="text-sm font-mono text-base-content">{props.children}</dd>
    </div>
  );
};

export const DisplayStateCard: Component<DisplayStateCardProps> = (props) => {
  const { t } = useLanguage();
  return (
    <div class="card bg-base-200 shadow-lg hover:shadow-xl transition-shadow duration-200">
      <div class="card-body p-4">
        <div class="card-title text-base mb-3 flex items-center justify-between">
          <span>{t('displays.title')}</span>
          <div class="badge badge-primary badge-outline">{t('common.realtime')}</div>
        </div>

        <div class="grid grid-cols-1 gap-3">
          {/* 亮度信息 */}
          <div class="bg-base-100 rounded-lg p-3">
            <h4 class="text-sm font-semibold text-base-content mb-2">{t('displays.brightnessSettings')}</h4>
            <div class="space-y-1">
              <Item label={t('displays.currentBrightness')}>{props.state.brightness}</Item>
              <Item label={t('displays.maxBrightness')}>{props.state.max_brightness}</Item>
              <Item label={t('displays.minBrightness')}>{props.state.min_brightness}</Item>
            </div>
          </div>

          {/* 对比度信息 */}
          <div class="bg-base-100 rounded-lg p-3">
            <h4 class="text-sm font-semibold text-base-content mb-2">{t('displays.contrastSettings')}</h4>
            <div class="space-y-1">
              <Item label={t('displays.currentContrast')}>{props.state.contrast}</Item>
              <Item label={t('displays.maxContrast')}>{props.state.max_contrast}</Item>
              <Item label={t('displays.minContrast')}>{props.state.min_contrast}</Item>
            </div>
          </div>

          {/* 模式信息 */}
          <div class="bg-base-100 rounded-lg p-3">
            <h4 class="text-sm font-semibold text-base-content mb-2">{t('displays.modeSettings')}</h4>
            <div class="space-y-1">
              <Item label={t('displays.currentMode')}>{props.state.mode}</Item>
              <Item label={t('displays.maxMode')}>{props.state.max_mode}</Item>
              <Item label={t('displays.minMode')}>{props.state.min_mode}</Item>
            </div>
          </div>

          {/* 更新时间 */}
          <div class="text-xs text-base-content/50 text-center pt-2 border-t border-base-300">
            {t('displays.lastModified')}: {props.state.last_modified_at.toLocaleString()}
          </div>
        </div>
      </div>
    </div>
  );
};
