import { Component, createSignal, createMemo, For, Show, onMount, createEffect, onCleanup } from 'solid-js';
import { useParams, useNavigate } from '@solidjs/router';
import { useLanguage } from '../../i18n/index';
import { LedColorService } from '../../services/led-color-service';
import { invoke } from '@tauri-apps/api/core';

// LED灯带配置类型
interface LedStripConfig {
  id: string;
  border: 'Top' | 'Bottom' | 'Left' | 'Right';
  count: number;
  reverse: boolean;
  ledType: 'WS2812B' | 'SK6812';
  driver: string;
  sequence: number;
  startOffset: number; // 0-100%
  endOffset: number;   // 0-100%
}



// 常量定义

// 默认配置
const DEFAULT_CONFIG = {
  longSide: 38,  // 长边LED数量
  shortSide: 22, // 短边LED数量
  ledType: 'WS2812B' as const,
  driver: 'Driver1',
};

// 颜色预览组件
const ColorPreview: Component<{ border: string; section: number }> = (props) => {
  // 8种不同的颜色，确保不重复
  const colors = ['#ff0000', '#00ff00', '#0000ff', '#ffff00', '#ff00ff', '#00ffff', '#ffa500', '#800080'];

  // 根据边框和分区计算唯一的颜色索引
  let colorIndex = 0;
  switch (props.border) {
    case 'Top':
      colorIndex = props.section; // 0, 1
      break;
    case 'Right':
      colorIndex = 2 + props.section; // 2, 3
      break;
    case 'Bottom':
      colorIndex = 4 + props.section; // 4, 5
      break;
    case 'Left':
      colorIndex = 6 + props.section; // 6, 7
      break;
  }

  const color = colors[colorIndex];

  return (
    <div
      class="absolute"
      style={{
        'background-color': color,
        ...(props.border === 'Top' || props.border === 'Bottom' ? {
          width: '50%',
          height: '8px',
          left: props.section === 0 ? '0%' : '50%',
          [props.border.toLowerCase()]: '0px'
        } : {
          width: '8px',
          height: '50%',
          top: props.section === 0 ? '0%' : '50%',
          [props.border.toLowerCase()]: '0px'
        })
      }}
    />
  );
};

// LED边框槽位组件 - 显示现有灯带
const LedBorderStrips: Component<{
  border: 'Top' | 'Bottom' | 'Left' | 'Right';
  strips: LedStripConfig[];
  onSelectStrip: (strip: LedStripConfig) => void;
  selectedStrip: LedStripConfig | null;
}> = (props) => {
  // 获取该边框的LED灯带
  const borderStrips = createMemo(() => {
    // 强制转换为字符串并去除空白字符
    const targetBorder = String(props.border).trim();

    const filtered = props.strips.filter(strip => {
      const stripBorder = String(strip.border).trim();
      return stripBorder === targetBorder;
    });

    return filtered;
  });

  // 为每个LED灯带生成独立的样式 - 条状平行显示
  const getStripStyle = (stripIndex: number, _totalStrips: number, strip: LedStripConfig, isSelected: boolean = false) => {
    const stripThickness = 8; // 灯带厚度
    const stripGap = 4;       // 灯带之间的间隙

    // 获取LED颜色服务
    const ledColorService = LedColorService.getInstance();

    // 生成该边框的预设颜色
    const borderColors = ledColorService.generateBorderColors(strip.border.toLowerCase(), strip.count);

    // 计算平均颜色用于显示
    const avgColor = borderColors.reduce(
      (acc, color) => ({
        r: acc.r + color.r / borderColors.length,
        g: acc.g + color.g / borderColors.length,
        b: acc.b + color.b / borderColors.length,
      }),
      { r: 0, g: 0, b: 0 }
    );

    // 应用基础亮度 - 为了UI可见性，使用更高的亮度
    const baseBrightness = 0.8; // 进一步提高亮度让灯带更明显
    const displayColor = `rgb(${Math.round(avgColor.r * baseBrightness)}, ${Math.round(avgColor.g * baseBrightness)}, ${Math.round(avgColor.b * baseBrightness)})`;

    const baseStyle = {
      position: 'absolute' as const,
      'z-index': isSelected ? '1000' : '999',
      cursor: 'pointer',
      transition: 'all 0.2s',
      'background-color': displayColor,
      'border-radius': '2px',
      border: isSelected
        ? '2px solid rgba(59, 130, 246, 0.8)' // 选中时蓝色边框
        : '1px solid rgba(255, 255, 255, 0.3)', // 默认白色边框
      'box-shadow': isSelected
        ? '0 4px 12px rgba(59, 130, 246, 0.4), 0 0 0 2px rgba(59, 130, 246, 0.2)' // 选中时蓝色发光效果
        : '0 1px 3px rgba(0, 0, 0, 0.3)', // 默认阴影
      transform: 'scale(1)', // 不缩放
      display: 'flex',
      'align-items': 'center',
      'justify-content': 'center',
    };

    // 让灯带与屏幕保持适当间距
    const baseOffset = 15; // 基础偏移距离，与屏幕保持间距
    const stripOffset = stripIndex * (stripThickness + stripGap);

    // 计算基于偏移量的位置和尺寸
    // startOffset: 灯带起始位置（从边缘开始的百分比）
    // endOffset: 灯带结束位置（从边缘开始的百分比）
    const startPercent = strip.startOffset / 100;
    const endPercent = strip.endOffset / 100;

    // 确保 endPercent >= startPercent，如果不是则交换
    const actualStart = Math.min(startPercent, endPercent);
    const actualEnd = Math.max(startPercent, endPercent);
    const actualLength = actualEnd - actualStart;



    switch (props.border) {
      case 'Top':
        return {
          ...baseStyle,
          top: `-${baseOffset + stripOffset}px`,
          left: `${actualStart * 100}%`,
          width: `${actualLength * 100}%`,
          height: `${stripThickness}px`,
        };
      case 'Bottom':
        return {
          ...baseStyle,
          bottom: `-${baseOffset + stripOffset}px`,
          left: `${actualStart * 100}%`,
          width: `${actualLength * 100}%`,
          height: `${stripThickness}px`,
        };
      case 'Left':
        return {
          ...baseStyle,
          left: `-${baseOffset + stripOffset}px`,
          top: `${actualStart * 100}%`,
          width: `${stripThickness}px`,
          height: `${actualLength * 100}%`,
        };
      case 'Right':
        return {
          ...baseStyle,
          right: `-${baseOffset + stripOffset}px`,
          top: `${actualStart * 100}%`,
          width: `${stripThickness}px`,
          height: `${actualLength * 100}%`,
        };
      default:
        return baseStyle;
    }
  };

  return (
    <Show
      when={borderStrips().length > 0}
      fallback={null}
    >
      <For each={borderStrips()}>
        {(strip, index) => {

          return (
            <div
              style={getStripStyle(index(), borderStrips().length, strip, props.selectedStrip?.id === strip.id)}
              onClick={() => {
                console.log('LED strip clicked:', strip.id, strip);
                props.onSelectStrip(strip);
              }}
              class="hover:brightness-110 transition-all duration-200"
            >
              <span style={{
                color: 'white',
                'font-size': '10px',
                'font-weight': 'bold',
                'text-shadow': '1px 1px 1px rgba(0,0,0,0.8)'
              }}>
                {strip.count}
              </span>
            </div>
          );
        }}
      </For>
    </Show>
  );
};



// LED边框添加按钮组件 - 在更外层显示
const LedBorderAddButton: Component<{
  border: 'Top' | 'Bottom' | 'Left' | 'Right';
  strips: LedStripConfig[];
  onCreateStrip: (border: 'Top' | 'Bottom' | 'Left' | 'Right') => void;
}> = (props) => {
  // 获取该边框的LED灯带数量
  const stripCount = createMemo(() =>
    props.strips.filter(strip => strip.border === props.border).length
  );

  const getAddButtonStyle = () => {
    const baseStyle = {
      position: 'absolute' as const,
      cursor: 'pointer',
      display: 'flex',
      'align-items': 'center',
      'justify-content': 'center',
      'font-size': '14px',
      transition: 'all 0.2s',
      'background-color': 'rgba(59, 130, 246, 0.1)',
      border: '2px dashed rgba(59, 130, 246, 0.3)',
      'border-radius': '4px',
      color: 'rgba(59, 130, 246, 0.7)',
      'z-index': '15', // 确保添加按钮在LED灯带之上
    };

    // 根据该边框的灯带数量动态计算偏移量
    // 与实际LED灯带渲染保持一致的参数
    const count = stripCount();
    const ledBaseOffset = 15; // LED灯带的基础偏移量（与getStripStyle一致）
    const stripThickness = 8; // 灯带厚度（与getStripStyle一致）
    const stripGap = 4; // 灯带间距（与getStripStyle一致）
    const buttonMargin = 20; // 按钮与最后一个灯带的间距，增加到20px

    // 计算：LED基础偏移 + 所有灯带占用的空间 + 按钮边距
    const offset = count > 0
      ? ledBaseOffset + (count * (stripThickness + stripGap)) + buttonMargin
      : ledBaseOffset + buttonMargin;

    switch (props.border) {
      case 'Top':
        return {
          ...baseStyle,
          top: `-${offset}px`,
          left: '50%',
          transform: 'translateX(-50%)',
          width: '120px',
          height: '24px',
        };
      case 'Bottom':
        return {
          ...baseStyle,
          bottom: `-${offset}px`,
          left: '50%',
          transform: 'translateX(-50%)',
          width: '120px',
          height: '24px',
        };
      case 'Left':
        return {
          ...baseStyle,
          left: `-${offset}px`,
          top: '50%',
          transform: 'translateY(-50%)',
          width: '24px',
          height: '60px',
          'writing-mode': 'vertical-rl' as const,
          'text-orientation': 'mixed' as const,
        };
      case 'Right':
        return {
          ...baseStyle,
          right: `-${offset}px`,
          top: '50%',
          transform: 'translateY(-50%)',
          width: '24px',
          height: '60px',
          'writing-mode': 'vertical-rl' as const,
          'text-orientation': 'mixed' as const,
        };
      default:
        return baseStyle;
    }
  };

  const getButtonText = () => {
    if (props.border === 'Left' || props.border === 'Right') {
      return '+';  // 纵向只显示加号
    }
    return stripCount() > 0 ? '+ 添加更多' : '+ 添加LED灯带';
  };

  return (
    <div
      style={getAddButtonStyle()}
      onClick={() => props.onCreateStrip(props.border)}
      title={`点击添加${props.border}边LED灯带`}
      class="hover:bg-blue-200 hover:border-blue-400"
    >
      {getButtonText()}
    </div>
  );
};

// LED配置面板组件
const LedConfigPanel: Component<{
  strip: LedStripConfig;
  onUpdate: (strip: LedStripConfig) => void;
  onDelete: (stripId: string) => void;
  availableDrivers: string[];
}> = (props) => {
  const { t } = useLanguage();

  const updateStrip = (updates: Partial<LedStripConfig>) => {
    props.onUpdate({ ...props.strip, ...updates });
  };

  return (
    <div class="card bg-base-100 shadow-lg">
      <div class="card-body p-4">
        <h3 class="card-title text-sm mb-4">
          {t('ledConfig.configPanel')} - {props.strip.border}
        </h3>

        {/* LED数量 */}
        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">{t('ledConfig.count')}</span>
          </label>
          <div class="flex items-center gap-2">
            <button
              class="btn btn-sm btn-circle"
              onClick={() => updateStrip({ count: Math.max(1, props.strip.count - 1) })}
            >
              -
            </button>
            <input
              type="number"
              class="input input-sm input-bordered flex-1 text-center"
              value={props.strip.count}
              onChange={(e) => updateStrip({ count: parseInt(e.currentTarget.value) || 1 })}
              min="1"
            />
            <button
              class="btn btn-sm btn-circle"
              onClick={() => updateStrip({ count: props.strip.count + 1 })}
            >
              +
            </button>
          </div>
        </div>

        {/* 数据方向 */}
        <div class="form-control">
          <label class="label cursor-pointer">
            <span class="label-text text-xs">{t('ledConfig.reverse')}</span>
            <input
              type="checkbox"
              class="toggle toggle-sm"
              checked={props.strip.reverse}
              onChange={(e) => updateStrip({ reverse: e.currentTarget.checked })}
            />
          </label>
        </div>

        {/* LED类型 */}
        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">{t('ledConfig.ledType')}</span>
          </label>
          <select
            class="select select-sm select-bordered"
            value={props.strip.ledType}
            onChange={(e) => updateStrip({ ledType: e.currentTarget.value as 'WS2812B' | 'SK6812' })}
          >
            <option value="WS2812B">WS2812B (RGB)</option>
            <option value="SK6812">SK6812 (RGBW)</option>
          </select>
        </div>

        {/* 驱动器 */}
        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">{t('ledConfig.driver')}</span>
          </label>
          <select
            class="select select-sm select-bordered"
            value={props.strip.driver}
            onChange={(e) => updateStrip({ driver: e.currentTarget.value })}
          >
            <For each={props.availableDrivers}>
              {(driver) => <option value={driver}>{driver}</option>}
            </For>
          </select>
        </div>

        {/* 序号 */}
        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">{t('ledConfig.sequence')}</span>
          </label>
          <input
            type="number"
            class="input input-sm input-bordered"
            value={props.strip.sequence}
            onChange={(e) => updateStrip({ sequence: parseInt(e.currentTarget.value) || 1 })}
            min="1"
          />
        </div>

        {/* 位置偏移 */}
        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">{t('ledConfig.startOffset')}</span>
          </label>
          <input
            type="range"
            class="range range-sm"
            min="0"
            max="100"
            value={props.strip.startOffset}
            onChange={(e) => updateStrip({ startOffset: parseInt(e.currentTarget.value) })}
          />
          <div class="text-xs text-center">{props.strip.startOffset}%</div>
        </div>

        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">{t('ledConfig.endOffset')}</span>
          </label>
          <input
            type="range"
            class="range range-sm"
            min="0"
            max="100"
            value={props.strip.endOffset}
            onChange={(e) => updateStrip({ endOffset: parseInt(e.currentTarget.value) })}
          />
          <div class="text-xs text-center">{props.strip.endOffset}%</div>
        </div>

        {/* 删除按钮 */}
        <div class="card-actions justify-end mt-4">
          <button
            class="btn btn-sm btn-error"
            onClick={() => props.onDelete(props.strip.id)}
          >
            {t('common.delete')}
          </button>
        </div>
      </div>
    </div>
  );
};

export function SingleDisplayConfig() {
  const params = useParams();
  const navigate = useNavigate();
  const { t } = useLanguage();

  const displayId = () => parseInt(params.displayId);

  // LED灯带配置状态
  const [ledStrips, setLedStrips] = createSignal<LedStripConfig[]>([]);
  const [selectedStrip, setSelectedStrip] = createSignal<LedStripConfig | null>(null);

  // 边框定义
  const borders: ('Top' | 'Bottom' | 'Left' | 'Right')[] = ['Top', 'Right', 'Bottom', 'Left'];

  // 可用驱动器列表
  const availableDrivers = ['Driver1', 'Driver2', 'Driver3'];

  // 保存LED灯带配置到后端
  const saveLedStripsToBackend = async (strips: LedStripConfig[]) => {
    try {
      console.log('=== 开始保存LED灯带配置 ===');
      console.log('显示器ID:', displayId());
      console.log('要保存的灯带数量:', strips.length);
      console.log('前端格式的灯带配置:', strips);

      // 转换为后端期望的格式
      const backendStrips = strips.map((strip, index) => ({
        index: index,
        border: strip.border,
        display_id: displayId(),
        start_pos: Math.floor(strip.startOffset * strip.count / 100),
        len: strip.count,
        led_type: strip.ledType === 'WS2812B' ? 'WS2812B' : 'SK6812',
        // 注意：后端可能不支持reverse字段，需要在颜色生成时处理
      }));

      console.log('转换为后端格式的配置:', backendStrips);

      await invoke('write_led_strip_configs', { configs: backendStrips });

      console.log('✅ 成功保存LED灯带配置到后端');
    } catch (error) {
      console.error('❌ 保存LED灯带配置失败:', error);
      throw error; // 重新抛出错误以便上层处理
    }
  };

  // 启用测试模式
  const startTestMode = async () => {
    try {
      console.log('Starting LED test mode...');
      await invoke('enable_test_mode');
      console.log('LED test mode enabled');
    } catch (error) {
      console.error('Failed to start test mode:', error);
    }
  };

  // 停止测试模式
  const stopTestMode = async () => {
    try {
      console.log('Stopping LED test mode...');
      await invoke('disable_test_mode');
      console.log('LED test mode disabled, ambient light resumed');
    } catch (error) {
      console.error('Failed to stop test mode:', error);
    }
  };

  // 加载LED灯带数据
  onMount(async () => {
    // 停止氛围光模式，启用测试模式
    await startTestMode();

    try {
      // 检查是否在 Tauri 环境中
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        console.log('=== 开始加载LED灯带配置 ===');
        console.log('显示器ID:', displayId());

        // 尝试从后端加载已保存的配置
        const allConfigs = await invoke('read_led_strip_configs');

        console.log('从后端加载的完整配置组:', allConfigs);
        console.log('配置组类型:', typeof allConfigs);

        // 从配置组中提取当前显示器的配置
        let savedConfigs = [];
        if (allConfigs && (allConfigs as any).strips && Array.isArray((allConfigs as any).strips)) {
          const currentDisplayId = displayId();
          savedConfigs = (allConfigs as any).strips.filter((config: any) => config.display_id === currentDisplayId);
          console.log('当前显示器ID:', currentDisplayId);
          console.log('所有灯带配置数量:', (allConfigs as any).strips.length);
          console.log('当前显示器的灯带配置:', savedConfigs);
        } else {
          console.log('配置组格式不正确或为空');
        }

        if (savedConfigs && Array.isArray(savedConfigs) && savedConfigs.length > 0) {
          // 转换后端数据为前端格式
          const convertedStrips: LedStripConfig[] = savedConfigs.map((config: any, index: number) => ({
            id: `strip-${config.border.toLowerCase()}-${index}`,
            border: config.border,
            count: config.len,
            ledType: config.led_type === 'SK6812' ? 'SK6812' : 'WS2812B',
            driver: 'Driver1', // 默认驱动器
            sequence: index + 1,
            startOffset: config.len > 0 ? Math.floor((config.start_pos / config.len) * 100) : 0,
            endOffset: 100, // 默认延伸到边缘末端
            reverse: false
          }));

          console.log('转换为前端格式的配置:', convertedStrips);
          console.log('转换后的灯带数量:', convertedStrips.length);

          setLedStrips(convertedStrips);

          if (convertedStrips.length > 0) {
            setSelectedStrip(convertedStrips[0]);
            console.log('设置默认选中的灯带:', convertedStrips[0].id);
          }

          console.log('✅ 成功加载已保存的LED灯带配置');

          // 立即启动30Hz测试颜色发送
          console.log('=== 立即启动测试颜色发送（已保存配置）===');
          setTimeout(() => {
            startTestColorSending();
          }, 100); // 稍微延迟确保状态已更新

          return; // 成功加载，不需要使用测试数据
        } else {
          console.log('No saved configuration found, using test data');
        }
      } else {
        throw new Error('Not in Tauri environment, using test data');
      }
    } catch (error) {
      console.log('Failed to load saved configuration, using test data:', error);
    }

    // 如果没有保存的配置或加载失败，使用测试数据
    console.log('Loading test data...');
      const testStrips: LedStripConfig[] = [
        {
          id: 'test-right-1',
          border: 'Right',
          count: 22,
          ledType: 'WS2812B',
          driver: 'Driver1',
          sequence: 2,
          startOffset: 5,
          endOffset: 84,
          reverse: false,
        },
        {
          id: 'test-right-2',
          border: 'Right',
          count: 18,
          ledType: 'WS2812B',
          driver: 'Driver1',
          sequence: 3,
          startOffset: 0,
          endOffset: 100,
          reverse: false,
        },
        {
          id: 'test-bottom-1',
          border: 'Bottom',
          count: 38,
          ledType: 'WS2812B',
          driver: 'Driver1',
          sequence: 4,
          startOffset: 0,
          endOffset: 100,
          reverse: false,
        },
        {
          id: 'test-bottom-2',
          border: 'Bottom',
          count: 32,
          ledType: 'WS2812B',
          driver: 'Driver1',
          sequence: 5,
          startOffset: 0,
          endOffset: 100,
          reverse: false,
        }
      ];
      setLedStrips(testStrips);
      setSelectedStrip(testStrips[0]);

      // 注意：测试数据不会自动保存，只有用户点击保存按钮才会保存
      console.log('Loaded test data (not saved automatically):', testStrips);

      // 立即启动30Hz测试颜色发送
      console.log('=== 立即启动测试颜色发送 ===');
      setTimeout(() => {
        startTestColorSending();
      }, 100); // 稍微延迟确保状态已更新
  });

  // 组件卸载时的清理
  onCleanup(() => {
    // 恢复氛围光模式
    stopTestMode();
  });

  // 创建新LED灯带
  const createLedStrip = (border: 'Top' | 'Bottom' | 'Left' | 'Right') => {
    const isLongSide = border === 'Top' || border === 'Bottom';
    const defaultCount = isLongSide ? DEFAULT_CONFIG.longSide : DEFAULT_CONFIG.shortSide;

    // 获取下一个序号
    const existingSequences = ledStrips()
      .filter(s => s.driver === DEFAULT_CONFIG.driver)
      .map(s => s.sequence);
    const nextSequence = existingSequences.length > 0 ? Math.max(...existingSequences) + 1 : 1;

    const newStrip: LedStripConfig = {
      id: `strip_${Date.now()}_${Math.random()}`,
      border: border,
      count: defaultCount,
      reverse: false,
      ledType: DEFAULT_CONFIG.ledType,
      driver: DEFAULT_CONFIG.driver,
      sequence: nextSequence,
      startOffset: 0,
      endOffset: 100, // 默认延伸到边缘末端
    };

    setLedStrips(prev => {
      const updated = [...prev, newStrip];
      return updated;
    });
    setSelectedStrip(newStrip);
  };

  // 更新LED灯带
  const updateLedStrip = (updatedStrip: LedStripConfig) => {
    setLedStrips(prev => prev.map(strip =>
      strip.id === updatedStrip.id ? updatedStrip : strip
    ));
    setSelectedStrip(updatedStrip);
  };

  // 删除LED灯带
  const deleteLedStrip = (stripId: string) => {
    setLedStrips(prev => prev.filter(strip => strip.id !== stripId));
    setSelectedStrip(null);
  };

  // 清空所有配置
  const clearAllConfig = () => {
    setLedStrips([]);
    setSelectedStrip(null);
  };

  // 保存配置状态
  const [isSaving, setIsSaving] = createSignal(false);
  const [saveStatus, setSaveStatus] = createSignal<'idle' | 'success' | 'error'>('idle');

  // 保存LED灯带配置
  const saveConfiguration = async () => {
    setIsSaving(true);
    setSaveStatus('idle');

    try {
      console.log('=== 开始保存配置 ===');
      console.log('当前要保存的配置:', ledStrips());

      // 保存到后端
      await saveLedStripsToBackend(ledStrips());

      // 验证保存：立即读取配置确认保存成功
      console.log('=== 验证保存结果 ===');
      try {
        const verifyAllConfigs = await invoke('read_led_strip_configs');
        console.log('保存后立即读取的完整配置:', verifyAllConfigs);

        // 过滤当前显示器的配置
        let verifyConfigs = [];
        if (verifyAllConfigs && (verifyAllConfigs as any).strips && Array.isArray((verifyAllConfigs as any).strips)) {
          const currentDisplayId = displayId();
          verifyConfigs = (verifyAllConfigs as any).strips.filter((config: any) => config.display_id === currentDisplayId);
          console.log('验证：当前显示器的配置数量:', verifyConfigs.length);
          console.log('验证：当前显示器的配置内容:', verifyConfigs);
        }

        if (verifyConfigs && Array.isArray(verifyConfigs) && verifyConfigs.length > 0) {
          console.log('✅ 验证成功：配置已正确保存');
        } else {
          console.log('⚠️ 验证警告：读取到的配置为空');
        }
      } catch (verifyError) {
        console.error('❌ 验证失败：无法读取保存的配置', verifyError);
      }

      // 显示成功状态
      setSaveStatus('success');
      console.log('✅ LED灯带配置保存完成');

      // 3秒后重置状态
      setTimeout(() => {
        setSaveStatus('idle');
      }, 3000);

    } catch (error) {
      console.error('❌ 保存LED灯带配置失败:', error);
      setSaveStatus('error');

      // 5秒后重置状态
      setTimeout(() => {
        setSaveStatus('idle');
      }, 5000);
    } finally {
      setIsSaving(false);
    }
  };

  // 生成边框预设颜色：每个边框被两个颜色平分
  const generateBorderTestColors = (border: string, ledCount: number, reverse: boolean = false) => {
    const colors = [];
    const halfCount = Math.floor(ledCount / 2);

    // 定义每个边框的两个颜色
    const borderColorPairs = {
      'top': [
        { r: 255, g: 0, b: 0 },    // 红色
        { r: 255, g: 165, b: 0 }   // 橙色
      ],
      'right': [
        { r: 0, g: 255, b: 0 },    // 绿色
        { r: 0, g: 255, b: 255 }   // 青色
      ],
      'bottom': [
        { r: 0, g: 0, b: 255 },    // 蓝色
        { r: 128, g: 0, b: 128 }   // 紫色
      ],
      'left': [
        { r: 255, g: 255, b: 0 },  // 黄色
        { r: 255, g: 0, b: 255 }   // 洋红色
      ]
    };

    const colorPair = borderColorPairs[border.toLowerCase() as keyof typeof borderColorPairs] || borderColorPairs['top'];

    // 前半部分使用第一个颜色
    for (let i = 0; i < halfCount; i++) {
      colors.push(colorPair[0]);
    }

    // 后半部分使用第二个颜色
    for (let i = halfCount; i < ledCount; i++) {
      colors.push(colorPair[1]);
    }

    // 如果设置了反向，则反转颜色数组
    if (reverse) {
      colors.reverse();
    }

    return colors;
  };

  // 30Hz测试颜色发送定时器
  let testColorTimer: number | null = null;

  // 发送测试颜色到单个灯带
  const sendTestColorsToStrip = async (strip: LedStripConfig) => {
    try {
      // 生成该边框的预设颜色（两个颜色平分，考虑反向设置）
      const borderColors = generateBorderTestColors(strip.border, strip.count, strip.reverse);

      // 发送到硬件
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        try {
          // 转换为硬件期望的格式
          let colorBytes = [];

          if (strip.ledType === 'SK6812') {
            // SK6812 RGBW格式：G,R,B,W
            colorBytes = borderColors.map(color => [color.g, color.r, color.b, 0]).flat();
          } else {
            // WS2812B RGB格式：G,R,B
            colorBytes = borderColors.map(color => [color.g, color.r, color.b]).flat();
          }

          const bytesPerLed = strip.ledType === 'SK6812' ? 4 : 3;

          // 计算偏移量：基于LED灯带的起始偏移量
          const ledOffset = Math.floor(strip.startOffset * strip.count / 100);
          const byteOffset = ledOffset * bytesPerLed;

          // 只在第一次发送时显示详细信息
          if (!testColorTimer || Math.random() < 0.001) { // 0.1%概率显示，避免日志过多
            console.log(`🚀 发送颜色: ${strip.id}, 偏移=${byteOffset}, 长度=${colorBytes.length}`);
          }

          // 获取第一个可用的设备地址
          const boards = await invoke('get_boards') as any[];
          if (boards.length === 0) {
            console.error('❌ 没有找到可用的设备');
            return;
          }

          const board = boards[0];
          const boardAddress = `${board.address}:${board.port}`;

          // 使用 send_test_colors_to_board 直接发送，不依赖设备连接状态
          await invoke('send_test_colors_to_board', {
            boardAddress: boardAddress,
            offset: byteOffset,
            buffer: colorBytes
          });

        } catch (error) {
          console.error('❌ 发送颜色到硬件失败:', strip.id, error);
        }
      }
    } catch (error) {
      console.error('Failed to generate test colors for strip:', strip.id, error);
    }
  };

  // 启动30Hz测试颜色发送
  const startTestColorSending = () => {
    if (testColorTimer) {
      clearInterval(testColorTimer);
    }

    const strips = ledStrips();
    console.log('=== 启动30Hz测试颜色发送 ===');
    console.log(`发送频率: 30Hz (每33.33ms发送一次)`);
    console.log(`目标灯带数量: ${strips.length}`);

    // 立即发送一次
    strips.forEach(strip => {
      sendTestColorsToStrip(strip);
    });

    let frameCount = 0;
    const startTime = Date.now();

    // 启动30Hz定时器 (1000ms / 30 = 33.33ms)
    testColorTimer = setInterval(() => {
      const currentStrips = ledStrips();
      if (currentStrips.length > 0) {
        currentStrips.forEach(strip => {
          sendTestColorsToStrip(strip);
        });

        frameCount++;
        // 每秒显示一次统计信息
        if (frameCount % 30 === 0) {
          const elapsed = (Date.now() - startTime) / 1000;
          const actualFps = frameCount / elapsed;
          console.log(`📊 30Hz发送统计: 已发送${frameCount}帧, 实际频率: ${actualFps.toFixed(1)}Hz`);
        }
      }
    }, 33) as any; // 30Hz = 33.33ms间隔
  };

  // 停止测试颜色发送
  const stopTestColorSending = () => {
    if (testColorTimer) {
      clearInterval(testColorTimer);
      testColorTimer = null;
      console.log('=== 停止30Hz测试颜色发送 ===');
    }
  };

  // 当灯带配置变化时，重新启动测试颜色发送
  createEffect(() => {
    const strips = ledStrips();
    // 通过访问每个灯带的所有属性来确保深度监听
    const stripSignature = strips.map(strip =>
      `${strip.id}-${strip.count}-${strip.reverse}-${strip.ledType}-${strip.startOffset}-${strip.endOffset}`
    ).join('|');

    if (strips.length > 0) {
      console.log(`=== 检测到${strips.length}个已配置的灯带，启动30Hz测试颜色发送 ===`);
      console.log(`配置签名: ${stripSignature}`);
      strips.forEach(strip => {
        console.log(`灯带: ${strip.id} (${strip.border}边) - ${strip.count}个LED, 反向: ${strip.reverse}`);
      });
      // 重新启动30Hz发送（这会处理所有配置变化）
      startTestColorSending();
    } else {
      console.log('=== 没有配置的灯带，停止测试颜色发送 ===');
      stopTestColorSending();
    }
  });

  // 清理效果：离开界面时停止所有LED效果
  onCleanup(() => {
    // 停止30Hz测试颜色发送
    stopTestColorSending();

    // 恢复氛围光模式
    stopTestMode();

    const ledColorService = LedColorService.getInstance();
    ledStrips().forEach((strip) => {
      ledColorService.stopBreathingEffect(strip.id);
    });
  });

  return (
    <div class="container mx-auto p-6 h-full">
      <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">{t('singleDisplayConfig.title')}</h1>
        <div class="flex gap-2 items-center">
          {/* 保存状态提示 */}
          <Show when={saveStatus() === 'success'}>
            <div class="text-success text-sm flex items-center mr-2">
              <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
              </svg>
              配置已保存
            </div>
          </Show>

          <Show when={saveStatus() === 'error'}>
            <div class="text-error text-sm flex items-center mr-2">
              <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
              保存失败
            </div>
          </Show>

          {/* 保存按钮 */}
          <button
            class="btn btn-primary"
            onClick={saveConfiguration}
            disabled={isSaving() || ledStrips().length === 0}
          >
            <Show when={isSaving()}>
              <span class="loading loading-spinner loading-sm mr-2"></span>
            </Show>
            <Show when={!isSaving()}>
              <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3-3m0 0l-3 3m3-3v12"></path>
              </svg>
            </Show>
            {isSaving() ? '保存中...' : '保存配置'}
          </button>

          <button
            class="btn btn-outline btn-error"
            onClick={clearAllConfig}
          >
            {t('common.clear')}
          </button>
          <button
            class="btn btn-outline"
            onClick={() => navigate('/led-strips-configuration')}
          >
            {t('common.back')}
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-4 gap-6 h-full">
        {/* 中央显示器区域 */}
        <div class="lg:col-span-3">
          <div class="card bg-base-200 shadow-lg h-full">
            <div class="card-body flex items-center justify-center">
              {/* LED配置区域容器 - 为LED灯带提供定位基准 */}
              <div class="relative">
                {/* 显示器主体 */}
                <div
                  class="bg-base-300 border-2 border-base-content/20 rounded relative"
                  style={{
                    width: '400px',
                    height: '300px',
                  }}
                >
                  {/* 颜色预览区域 */}
                  <ColorPreview border="Top" section={0} />
                  <ColorPreview border="Top" section={1} />
                  <ColorPreview border="Right" section={0} />
                  <ColorPreview border="Right" section={1} />
                  <ColorPreview border="Bottom" section={0} />
                  <ColorPreview border="Bottom" section={1} />
                  <ColorPreview border="Left" section={0} />
                  <ColorPreview border="Left" section={1} />

                  {/* 显示器信息 */}
                  <div class="absolute inset-0 flex items-center justify-center">
                    <div class="text-center">
                      <div class="font-semibold">Display {displayId()}</div>
                      <div class="text-sm text-base-content/60">LED Configuration</div>
                    </div>
                  </div>
                </div>

                {/* LED边框现有灯带显示 */}
                <For each={borders}>
                  {(border) => (
                    <LedBorderStrips
                      border={border}
                      strips={ledStrips()}
                      onSelectStrip={(strip) => {
                        console.log('Setting selected strip:', strip.id, strip);
                        setSelectedStrip(strip);
                        console.log('Selected strip after set:', selectedStrip());
                      }}
                      selectedStrip={selectedStrip()}
                    />
                  )}
                </For>

                {/* LED边框添加按钮 - 相对于显示器定位 */}
                <For each={borders}>
                  {(border) => (
                    <LedBorderAddButton
                      border={border}
                      strips={ledStrips()}
                      onCreateStrip={createLedStrip}
                    />
                  )}
                </For>
              </div>
            </div>
          </div>
        </div>

        {/* 右侧配置面板 */}
        <div class="lg:col-span-1">
          <Show
            when={selectedStrip()}
            fallback={
              <div class="card bg-base-100 shadow-lg">
                <div class="card-body text-center text-base-content/60">
                  <p>{t('singleDisplayConfig.selectOrCreateStrip')}</p>
                  <p class="text-xs mt-2">当前选中: {selectedStrip() ? selectedStrip()!.id : '无'}</p>
                  <p class="text-xs">总灯带数: {ledStrips().length}</p>
                </div>
              </div>
            }
          >
            <LedConfigPanel
              strip={selectedStrip()!}
              onUpdate={updateLedStrip}
              onDelete={deleteLedStrip}
              availableDrivers={availableDrivers}
            />
          </Show>
        </div>
      </div>
    </div>
  );
};
