import { Component, createSignal } from 'solid-js';

const ColorItem: Component<{
  color: string;
  position: [number, number];
  size?: [number, number];
  onClick?: (color: string) => void;
}> = (props) => {
  return (
    <div
      style={{
        background: props.color,
        'grid-row-start': props.position[0],
        'grid-column-start': props.position[1],
        'grid-row-end': props.position[0] + (props.size ? props.size[0] : 1),
        'grid-column-end': props.position[1] + (props.size ? props.size[1] : 1),
        cursor: props.onClick ? 'pointer' : 'default',
      }}
      onClick={() => {
        props.onClick?.(props.color);
      }}
      title={props.color}
    />
  );
};

export const TestColorsBg: Component = () => {
  const [singleColor, setSingleColor] = createSignal<string | null>(null);

  return (
    <>
      <section
        class="grid grid-cols-[8] grid-rows-[8] h-full w-full"
        classList={{
          hidden: singleColor() !== null,
        }}
      >
        <ColorItem color="#ff0000" position={[1, 1]} onClick={setSingleColor} />
        <ColorItem color="#ffff00" position={[1, 2]} onClick={setSingleColor} />
        <ColorItem color="#00ff00" position={[1, 3]} onClick={setSingleColor} />
        <ColorItem color="#00ffff" position={[1, 4]} onClick={setSingleColor} />
        <ColorItem color="#0000ff" position={[1, 5]} onClick={setSingleColor} />
        <ColorItem color="#ff00ff" position={[1, 6]} onClick={setSingleColor} />
        <ColorItem color="#ffffff" position={[1, 7]} onClick={setSingleColor} />
        <ColorItem color="#000000" position={[1, 8]} onClick={setSingleColor} />
        <ColorItem color="#ffff00" position={[2, 1]} onClick={setSingleColor} />
        <ColorItem color="#00ff00" position={[3, 1]} onClick={setSingleColor} />
        <ColorItem color="#00ffff" position={[4, 1]} onClick={setSingleColor} />
        <ColorItem color="#0000ff" position={[5, 1]} onClick={setSingleColor} />
        <ColorItem color="#ff00ff" position={[6, 1]} onClick={setSingleColor} />
        <ColorItem color="#ffffff" position={[7, 1]} onClick={setSingleColor} />
        <ColorItem color="#000000" position={[8, 1]} onClick={setSingleColor} />
        <ColorItem color="#ffffff" position={[2, 8]} onClick={setSingleColor} />
        <ColorItem color="#ff00ff" position={[3, 8]} onClick={setSingleColor} />
        <ColorItem color="#0000ff" position={[4, 8]} onClick={setSingleColor} />
        <ColorItem color="#00ffff" position={[5, 8]} onClick={setSingleColor} />
        <ColorItem color="#00ff00" position={[6, 8]} onClick={setSingleColor} />
        <ColorItem color="#ffff00" position={[7, 8]} onClick={setSingleColor} />
        <ColorItem color="#ff0000" position={[8, 8]} onClick={setSingleColor} />
        <ColorItem color="#ffffff" position={[8, 2]} onClick={setSingleColor} />
        <ColorItem color="#ff00ff" position={[8, 3]} onClick={setSingleColor} />
        <ColorItem color="#0000ff" position={[8, 4]} onClick={setSingleColor} />
        <ColorItem color="#00ffff" position={[8, 5]} onClick={setSingleColor} />
        <ColorItem color="#00ff00" position={[8, 6]} onClick={setSingleColor} />
        <ColorItem color="#ffff00" position={[8, 7]} onClick={setSingleColor} />
      </section>
      <section
        class="grid grid-cols-[8] grid-rows-[8] h-full w-full"
        classList={{
          hidden: singleColor() === null,
        }}
      >
        <ColorItem
          color={singleColor()!}
          position={[1, 1]}
          size={[1, 7]}
          onClick={() => setSingleColor(null)}
        />
        <ColorItem
          color={singleColor()!}
          position={[8, 2]}
          size={[1, 7]}
          onClick={() => setSingleColor(null)}
        />
        <ColorItem
          color={singleColor()!}
          position={[2, 1]}
          size={[7, 1]}
          onClick={() => setSingleColor(null)}
        />
        <ColorItem
          color={singleColor()!}
          position={[1, 8]}
          size={[7, 1]}
          onClick={() => setSingleColor(null)}
        />
      </section>
    </>
  );
};
