import { Component, ParentComponent } from 'solid-js';
import { DisplayState } from '../../models/display-state.model';

type DisplayStateCardProps = {
  state: DisplayState;
};

type ItemProps = {
  label: string;
};

const Item: ParentComponent<ItemProps> = (props) => {
  return (
    <dl class="flex">
      <dt class="w-20">{props.label}</dt>
      <dd class="flex-auto">{props.children}</dd>
    </dl>
  );
};

export const DisplayStateCard: Component<DisplayStateCardProps> = (props) => {
  return (
    <section class="p-2 rounded shadow">
      <Item label="Brightness">{props.state.brightness}</Item>
      <Item label="Max Brightness">{props.state.max_brightness}</Item>
      <Item label="Min Brightness">{props.state.min_brightness}</Item>
      <Item label="Contrast">{props.state.contrast}</Item>
      <Item label="Max Contrast">{props.state.max_contrast}</Item>
      <Item label="Min Contrast">{props.state.min_contrast}</Item>
      <Item label="Max Mode">{props.state.max_mode}</Item>
      <Item label="Min Mode">{props.state.min_mode}</Item>
      <Item label="Mode">{props.state.mode}</Item>
      <Item label="Last Modified At">{props.state.last_modified_at.toISOString()}</Item>
    </section>
  );
};
