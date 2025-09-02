import { Component, createSignal } from 'solid-js';
import { colorCalibrationService } from '../../services/color-calibration.service';

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

  // 处理颜色点击事件
  const handleColorClick = async (color: string) => {
    try {
      // 设置UI显示的颜色
      setSingleColor(color);

      // 应用颜色到LED硬件
      await colorCalibrationService.applyColorToAllLeds(color);
      console.log('🎨 颜色已应用:', color);
    } catch (error) {
      console.error('❌ 应用颜色失败:', error);
    }
  };

  // 处理清除颜色事件
  const handleClearColor = async () => {
    try {
      // 清除UI显示的颜色
      setSingleColor(null);

      // 清除LED硬件颜色
      await colorCalibrationService.clearAllLeds();
      console.log('🧹 LED颜色已清除');
    } catch (error) {
      console.error('❌ 清除LED颜色失败:', error);
    }
  };

  return (
    <>
      <section
        class="color-test-grid"
        classList={{
          hidden: singleColor() !== null,
        }}
      >
        <ColorItem color="#ff0000" position={[1, 1]} onClick={handleColorClick} />
        <ColorItem color="#ffff00" position={[1, 2]} onClick={handleColorClick} />
        <ColorItem color="#00ff00" position={[1, 3]} onClick={handleColorClick} />
        <ColorItem color="#00ffff" position={[1, 4]} onClick={handleColorClick} />
        <ColorItem color="#0000ff" position={[1, 5]} onClick={handleColorClick} />
        <ColorItem color="#ff00ff" position={[1, 6]} onClick={handleColorClick} />
        <ColorItem color="#ffffff" position={[1, 7]} onClick={handleColorClick} />
        <ColorItem color="#000000" position={[1, 8]} onClick={handleColorClick} />
        <ColorItem color="#ffff00" position={[2, 1]} onClick={handleColorClick} />
        <ColorItem color="#00ff00" position={[3, 1]} onClick={handleColorClick} />
        <ColorItem color="#00ffff" position={[4, 1]} onClick={handleColorClick} />
        <ColorItem color="#0000ff" position={[5, 1]} onClick={handleColorClick} />
        <ColorItem color="#ff00ff" position={[6, 1]} onClick={handleColorClick} />
        <ColorItem color="#ffffff" position={[7, 1]} onClick={handleColorClick} />
        <ColorItem color="#000000" position={[8, 1]} onClick={handleColorClick} />
        <ColorItem color="#ffffff" position={[2, 8]} onClick={handleColorClick} />
        <ColorItem color="#ff00ff" position={[3, 8]} onClick={handleColorClick} />
        <ColorItem color="#0000ff" position={[4, 8]} onClick={handleColorClick} />
        <ColorItem color="#00ffff" position={[5, 8]} onClick={handleColorClick} />
        <ColorItem color="#00ff00" position={[6, 8]} onClick={handleColorClick} />
        <ColorItem color="#ffff00" position={[7, 8]} onClick={handleColorClick} />
        <ColorItem color="#ff0000" position={[8, 8]} onClick={handleColorClick} />
        <ColorItem color="#ffffff" position={[8, 2]} onClick={handleColorClick} />
        <ColorItem color="#ff00ff" position={[8, 3]} onClick={handleColorClick} />
        <ColorItem color="#0000ff" position={[8, 4]} onClick={handleColorClick} />
        <ColorItem color="#00ffff" position={[8, 5]} onClick={handleColorClick} />
        <ColorItem color="#00ff00" position={[8, 6]} onClick={handleColorClick} />
        <ColorItem color="#ffff00" position={[8, 7]} onClick={handleColorClick} />
      </section>
      <section
        class="color-test-grid"
        classList={{
          hidden: singleColor() === null,
        }}
      >
        <ColorItem
          color={singleColor()!}
          position={[1, 1]}
          size={[1, 7]}
          onClick={handleClearColor}
        />
        <ColorItem
          color={singleColor()!}
          position={[8, 1]}
          size={[1, 8]}
          onClick={handleClearColor}
        />
        <ColorItem
          color={singleColor()!}
          position={[2, 1]}
          size={[7, 1]}
          onClick={handleClearColor}
        />
        <ColorItem
          color={singleColor()!}
          position={[1, 8]}
          size={[8, 1]}
          onClick={handleClearColor}
        />
      </section>
    </>
  );
};
