import { Component, JSX } from 'solid-js';

type Props = {
  value?: number;
} & JSX.HTMLAttributes<HTMLInputElement>;

export const ColorSlider: Component<Props> = (props) => {
  const handleMouseDown = (e: MouseEvent) => {
    // 阻止事件冒泡到父元素，避免触发面板拖拽
    e.stopPropagation();
  };

  const handleMouseMove = (e: MouseEvent) => {
    // 阻止事件冒泡到父元素
    e.stopPropagation();
  };

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
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
    />
  );
};
