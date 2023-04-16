import { Component, JSX } from 'solid-js';

type Props = {} & JSX.HTMLAttributes<HTMLInputElement>;

export const ColorSlider: Component<Props> = (props) => {
  return (
    <input
      type="range"
      value="50"
      {...props}
      class={
        'w-full h-2 bg-gradient-to-r rounded-lg appearance-none cursor-pointer dark:bg-gray-700 drop-shadow ' +
        props.class
      }
    />
  );
};
