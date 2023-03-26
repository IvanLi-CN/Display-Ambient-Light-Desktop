import { Component, JSX, ParentComponent, splitProps } from 'solid-js';
import { DisplayInfo } from '../models/display-info.model';

type DisplayInfoItemProps = {
  label: string;
};

export const DisplayInfoItem: ParentComponent<DisplayInfoItemProps> = (props) => {
  return (
    <dl class="px-3 py-1 flex hover:bg-slate-900/50 gap-2 text-white drop-shadow-[0_2px_2px_rgba(0,0,0,0.8)] rounded">
      <dt class="uppercase w-1/2 select-all whitespace-nowrap">{props.label}</dt>
      <dd class="select-all w-1/2 whitespace-nowrap">{props.children}</dd>
    </dl>
  );
};

type DisplayInfoPanelProps = {
  display: DisplayInfo;
} & JSX.HTMLAttributes<HTMLElement>;

export const DisplayInfoPanel: Component<DisplayInfoPanelProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['display']);
  return (
    <section {...rootProps} class={'m-2 flex flex-col gap-1 py-2 ' + rootProps.class}>
      <DisplayInfoItem label="ID">
        <code>{localProps.display.id}</code>
      </DisplayInfoItem>
      <DisplayInfoItem label="Position">
        ({localProps.display.x}, {localProps.display.y})
      </DisplayInfoItem>
      <DisplayInfoItem label="Size">
        {localProps.display.width} x {localProps.display.height}
      </DisplayInfoItem>
      <DisplayInfoItem label="Scale Factor">
        {localProps.display.scale_factor}
      </DisplayInfoItem>
      <DisplayInfoItem label="is Primary">
        {localProps.display.is_primary ? 'True' : 'False'}
      </DisplayInfoItem>
    </section>
  );
};
