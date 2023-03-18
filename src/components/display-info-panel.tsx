import { Component, ParentComponent } from 'solid-js';
import { DisplayInfo } from '../models/display-info.model';

type DisplayInfoItemProps = {
  label: string;
};

export const DisplayInfoItem: ParentComponent<DisplayInfoItemProps> = (props) => {
  return (
    <dl class="m-1 flex hover:bg-gray-100 gap-2 text-gray-700">
      <dt class="uppercase w-1/3">{props.label}</dt>
      <dd>{props.children}</dd>
    </dl>
  );
};

type DisplayInfoPanelProps = {
  display: DisplayInfo;
};

export const DisplayInfoPanel: Component<DisplayInfoPanelProps> = (props) => {
  return (
    <section class="m-2">
      <DisplayInfoItem label="ID">
        <code>{props.display.id}</code>
      </DisplayInfoItem>
      <DisplayInfoItem label="Position">
        ({props.display.x}, {props.display.y})
      </DisplayInfoItem>
      <DisplayInfoItem label="Size">
        {props.display.width} x {props.display.height}
      </DisplayInfoItem>
      <DisplayInfoItem label="Scale Factor">{props.display.scale_factor}</DisplayInfoItem>
      <DisplayInfoItem label="is Primary">
        {props.display.is_primary ? 'True' : 'False'}
      </DisplayInfoItem>
    </section>
  );
};
