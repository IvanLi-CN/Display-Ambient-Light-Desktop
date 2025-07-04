import { Component, JSX, ParentComponent, splitProps } from 'solid-js';
import { DisplayInfo } from '../../models/display-info.model';

type DisplayInfoItemProps = {
  label: string;
};

export const DisplayInfoItem: ParentComponent<DisplayInfoItemProps> = (props) => {
  return (
    <div class="flex justify-between items-center py-1 px-2 hover:bg-base-300/50 rounded transition-colors">
      <dt class="text-sm font-medium text-base-content/80 uppercase">{props.label}</dt>
      <dd class="text-sm font-mono text-base-content select-all">{props.children}</dd>
    </div>
  );
};

type DisplayInfoPanelProps = {
  display: DisplayInfo;
} & JSX.HTMLAttributes<HTMLElement>;

export const DisplayInfoPanel: Component<DisplayInfoPanelProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['display']);
  return (
    <div {...rootProps} class={'card bg-base-100/95 backdrop-blur shadow-lg border border-base-300 ' + rootProps.class}>
      <div class="card-body p-4">
        <div class="card-title text-sm mb-3 flex items-center justify-between">
          <span class="text-base-content">显示器信息</span>
          {localProps.display.is_primary && (
            <div class="badge badge-primary badge-sm">主显示器</div>
          )}
        </div>
        <div class="space-y-1">
          <DisplayInfoItem label="ID">
            <code class="bg-base-200 px-1 rounded text-xs">{localProps.display.id}</code>
          </DisplayInfoItem>
          <DisplayInfoItem label="位置">
            ({localProps.display.x}, {localProps.display.y})
          </DisplayInfoItem>
          <DisplayInfoItem label="尺寸">
            {localProps.display.width} × {localProps.display.height}
          </DisplayInfoItem>
          <DisplayInfoItem label="缩放">
            {localProps.display.scale_factor}×
          </DisplayInfoItem>
        </div>
      </div>
    </div>
  );
};
