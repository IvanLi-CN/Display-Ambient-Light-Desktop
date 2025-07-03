import { Component, JSX } from 'solid-js';

type Props = {
  value?: number;
} & JSX.HTMLAttributes<HTMLInputElement>;

export const ColorSlider: Component<Props> = (props) => {
  return (
    <input
      type="range"
      {...props}
      max={1}
      min={0}
      step={0.01}
      value={props.value}
      class={
        'range range-primary w-full bg-gradient-to-r ' +
        props.class
      }
    />
  );
};
