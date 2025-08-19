import { Component, createSignal, createMemo, For, Show, onMount, createEffect, onCleanup } from 'solid-js';
import { useParams, useNavigate } from '@solidjs/router';
import { useLanguage } from '../../i18n/index';
import { LedColorService } from '../../services/led-color-service';
import { adaptiveApi } from '../../services/api-adapter';
import { WebSocketListener } from '../websocket-listener';

// LED灯带配置类型
interface LedStripConfig {
  id: string;
  displayId: number; // Add displayId to the interface
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

// HSV到RGB转换函数（用于颜色预览）
const hsvToRgbPreview = (h: number, s: number, v: number): { r: number; g: number; b: number } => {
  const c = v * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;

  let r_prime = 0, g_prime = 0, b_prime = 0;

  if (h < 60) {
    r_prime = c; g_prime = x; b_prime = 0;
  } else if (h < 120) {
    r_prime = x; g_prime = c; b_prime = 0;
  } else if (h < 180) {
    r_prime = 0; g_prime = c; b_prime = x;
  } else if (h < 240) {
    r_prime = 0; g_prime = x; b_prime = c;
  } else if (h < 300) {
    r_prime = x; g_prime = 0; b_prime = c;
  } else {
    r_prime = c; g_prime = 0; b_prime = x;
  }

  return {
    r: Math.round((r_prime + m) * 255),
    g: Math.round((g_prime + m) * 255),
    b: Math.round((b_prime + m) * 255)
  };
};

// 颜色预览组件
const ColorPreview: Component<{ border: string; section: number }> = (props) => {
  // 色环每45度的颜色定义 (HSV: H=色相, S=1.0, V=1.0)
  const colorWheel45Degrees = [
    hsvToRgbPreview(0, 1.0, 1.0),    // 0° - 红色
    hsvToRgbPreview(45, 1.0, 1.0),   // 45° - 橙色
    hsvToRgbPreview(90, 1.0, 1.0),   // 90° - 黄色
    hsvToRgbPreview(135, 1.0, 1.0),  // 135° - 黄绿色
    hsvToRgbPreview(180, 1.0, 1.0),  // 180° - 青色
    hsvToRgbPreview(225, 1.0, 1.0),  // 225° - 蓝色
    hsvToRgbPreview(270, 1.0, 1.0),  // 270° - 紫色
    hsvToRgbPreview(315, 1.0, 1.0),  // 315° - 玫红色
  ];

  // 定义每个边框的两个颜色 - 按色环45度间隔分配
  const borderColorPairs = {
    'Bottom': [
      colorWheel45Degrees[0],  // 红色 (0°)
      colorWheel45Degrees[1]   // 橙色 (45°)
    ],
    'Right': [
      colorWheel45Degrees[2],  // 黄色 (90°)
      colorWheel45Degrees[3]   // 黄绿色 (135°)
    ],
    'Top': [
      colorWheel45Degrees[4],  // 青色 (180°)
      colorWheel45Degrees[5]   // 蓝色 (225°)
    ],
    'Left': [
      colorWheel45Degrees[6],  // 紫色 (270°)
      colorWheel45Degrees[7]   // 玫红色 (315°)
    ]
  };

  const colorPair = borderColorPairs[props.border as keyof typeof borderColorPairs] || borderColorPairs['Top'];
  const selectedColor = colorPair[props.section]; // section 0 或 1
  const color = `rgb(${selectedColor.r}, ${selectedColor.g}, ${selectedColor.b})`;

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
  hoveredStrip: LedStripConfig | null;
  onHoverStrip: (strip: LedStripConfig | null) => void;
}> = (props) => {
  // 获取该边框的LED灯带
  const borderStrips = createMemo(() => {
    // 安全检查：确保 strips 存在且是数组
    if (!props.strips || !Array.isArray(props.strips)) {
      return [];
    }

    // 强制转换为字符串并去除空白字符
    const targetBorder = String(props.border).trim();

    const filtered = props.strips.filter(strip => {
      const stripBorder = String(strip.border).trim();
      return stripBorder === targetBorder;
    });

    return filtered;
  });

  // 为每个LED灯带生成独立的样式 - 条状平行显示
  const getStripStyle = (stripIndex: number, _totalStrips: number, strip: LedStripConfig, isSelected: boolean = false, isHovered: boolean = false) => {
    const stripThickness = 8; // 灯带厚度
    const stripGap = 4;       // 灯带之间的间隙

    // 所有灯带使用统一的颜色显示 - 不显示测试颜色的差异
    // 使用一个中性的LED灯带颜色，表示这是LED灯带的示意
    const uniformColor = { r: 255, g: 140, b: 0 }; // 橙色，代表LED灯带

    // 应用基础亮度 - 为了UI可见性，使用更高的亮度
    const baseBrightness = 0.8; // 进一步提高亮度让灯带更明显
    const displayColor = `rgb(${Math.round(uniformColor.r * baseBrightness)}, ${Math.round(uniformColor.g * baseBrightness)}, ${Math.round(uniformColor.b * baseBrightness)})`;

    // 根据状态确定样式
    let borderStyle, boxShadowStyle, zIndex;

    if (isSelected) {
      // 选中状态：蓝色边框和发光效果
      borderStyle = '2px solid rgba(59, 130, 246, 0.8)';
      boxShadowStyle = '0 4px 12px rgba(59, 130, 246, 0.4), 0 0 0 2px rgba(59, 130, 246, 0.2)';
      zIndex = '1001';
    } else if (isHovered) {
      // 悬浮状态：绿色边框和发光效果
      borderStyle = '2px solid rgba(34, 197, 94, 0.8)';
      boxShadowStyle = '0 4px 12px rgba(34, 197, 94, 0.4), 0 0 0 2px rgba(34, 197, 94, 0.2)';
      zIndex = '1000';
    } else {
      // 默认状态
      borderStyle = '1px solid rgba(255, 255, 255, 0.3)';
      boxShadowStyle = '0 1px 3px rgba(0, 0, 0, 0.3)';
      zIndex = '999';
    }

    const baseStyle = {
      position: 'absolute' as const,
      'z-index': zIndex,
      cursor: 'pointer',
      transition: 'all 0.2s',
      'background-color': displayColor,
      'border-radius': '2px',
      border: borderStyle,
      'box-shadow': boxShadowStyle,
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
          const isSelected = props.selectedStrip?.id === strip.id;
          const isHovered = props.hoveredStrip?.id === strip.id;

          return (
            <div
              style={getStripStyle(index(), borderStrips().length, strip, isSelected, isHovered)}
              onClick={() => {
                console.log('LED strip clicked:', strip.id, strip);
                props.onSelectStrip(strip);
              }}
              onMouseEnter={() => {
                console.log('LED strip hovered:', strip.id, strip);
                props.onHoverStrip(strip);
              }}
              onMouseLeave={() => {
                console.log('LED strip hover ended:', strip.id);
                props.onHoverStrip(null);
              }}
              class="transition-all duration-200"
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
  onCreateStrip: (border: 'Top' | 'Bottom' | 'Left' | 'Right') => Promise<void>;
}> = (props) => {
  // 获取该边框的LED灯带数量
  const stripCount = createMemo(() => {
    // 安全检查：确保 strips 存在且是数组
    if (!props.strips || !Array.isArray(props.strips)) {
      return 0;
    }
    return props.strips.filter(strip => strip.border === props.border).length;
  });

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
      onClick={async () => await props.onCreateStrip(props.border)}
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
  onUpdate: (strip: LedStripConfig) => Promise<void>;
  onDelete: (stripId: string) => Promise<void>;
  availableDrivers: string[];
}> = (props) => {
  const { t } = useLanguage();

  const updateStrip = async (updates: Partial<LedStripConfig>) => {
    await props.onUpdate({ ...props.strip, ...updates });
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
              onChange={async (e) => {
                const newReverseState = e.currentTarget.checked;

                try {
                  // 立即调用API反转LED灯带
                  await adaptiveApi.reverseLedStrip(props.strip.displayId, props.strip.border);

                  // API调用成功后更新本地状态
                  updateStrip({ reverse: newReverseState });

                  console.log(`✅ LED灯带反转成功: 显示器${props.strip.displayId} ${props.strip.border}边 -> ${newReverseState}`);
                } catch (error) {
                  console.error('❌ LED灯带反转失败:', error);

                  // 如果API调用失败，恢复开关状态
                  e.currentTarget.checked = !newReverseState;

                  // 可以在这里添加用户提示
                  // TODO: 显示错误提示给用户
                }
              }}
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
            onClick={async () => await props.onDelete(props.strip.id)}
          >
            {t('common.delete')}
          </button>
        </div>
      </div>
    </div>
  );
};

export function SingleDisplayConfig() {
  console.log('🎯 SingleDisplayConfig component is rendering');

  // 立即通过API报告组件渲染状态
  adaptiveApi.reportCurrentPage('🎯 SingleDisplayConfig 组件开始渲染')
    .catch((e: any) => console.error('Failed to report component render:', e));

  const params = useParams();
  const navigate = useNavigate();
  const { t } = useLanguage();

  console.log('🔍 SingleDisplayConfig - URL params:', params);

  const displayId = () => {
    const paramValue = params.displayId;
    const id = parseInt(paramValue || '1');
    console.log('🔍 SingleDisplayConfig - URL:', window.location.href);
    console.log('🔍 SingleDisplayConfig - displayId param value:', paramValue, 'type:', typeof paramValue);
    console.log('🔍 SingleDisplayConfig - parsed id:', id, 'isNaN:', isNaN(id));

    if (isNaN(id)) {
      console.error('❌ Invalid displayId parameter:', paramValue, 'defaulting to 1');
      return 1; // 默认返回显示器1
    }
    return id;
  };

  // LED灯带配置状态
  const [ledStrips, setLedStrips] = createSignal<LedStripConfig[]>([]);
  const [selectedStrip, setSelectedStrip] = createSignal<LedStripConfig | null>(null);
  const [hoveredStrip, setHoveredStrip] = createSignal<LedStripConfig | null>(null);

  // 边框定义
  const borders: ('Top' | 'Bottom' | 'Left' | 'Right')[] = ['Top', 'Right', 'Bottom', 'Left'];

  // 可用驱动器列表
  const availableDrivers = ['Driver1', 'Driver2', 'Driver3'];





  // 加载LED灯带数据
  onMount(async () => {
    if (import.meta.env.DEV) {
      console.log('onMount 开始执行');
    }

    try {
      // 总是尝试加载配置，不管是否在 Tauri 环境中
      if (import.meta.env.DEV) {
        console.log('开始加载LED灯带配置，显示器ID:', displayId());
      }

      // 获取显示器配置列表，用于 system displayId -> internal_id 映射
      const displayConfigs = await adaptiveApi.getDisplayConfigs();
      const currentDisplayId = displayId();
      const currentDisplay = Array.isArray(displayConfigs)
        ? displayConfigs.find((d: any) => d.last_system_id === currentDisplayId)
        : undefined;
      const targetInternalId = currentDisplay ? currentDisplay.internal_id : undefined;

      // 尝试从后端加载已保存的配置（V2）
      const v2Group = await adaptiveApi.readLedStripConfigs();

      console.log('从后端加载的V2配置组:', v2Group);

      // 从配置组中提取当前显示器（internal_id）的配置
      let savedConfigs = [] as any[];
      if (v2Group && (v2Group as any).strips && Array.isArray((v2Group as any).strips) && targetInternalId) {
        savedConfigs = (v2Group as any).strips.filter((config: any) => config.display_internal_id === targetInternalId);
        console.log('当前显示器 internal_id:', targetInternalId);
        console.log('所有灯带配置数量:', (v2Group as any).strips.length);
        console.log('当前显示器的灯带配置:', savedConfigs);
      } else {
        console.log('V2 配置组格式不正确或未找到对应 internal_id');
      }

        if (savedConfigs && Array.isArray(savedConfigs) && savedConfigs.length > 0) {
          // 转换后端 V2 数据为前端格式
          const currentDisplayIdNum = currentDisplayId;
          const convertedStrips: LedStripConfig[] = savedConfigs.map((config: any) => {
            return {
              id: `strip-${config.border.toLowerCase()}-${config.index}`,
              displayId: currentDisplayIdNum, // 前端仍用系统数值ID表示当前显示器
              border: config.border,
              count: config.len,
              ledType: config.led_type, // 直接映射
              driver: 'Driver1', // 默认驱动器
              sequence: config.index, // 直接使用后端的 index 作为 sequence
              startOffset: 0, // 保持用户设置的值，不要自动计算
              endOffset: 100, // 默认延伸到边缘末端
              reverse: config.reversed || false // 使用后端的 reversed 字段
            };
          });

          console.log('转换为前端格式的配置:', convertedStrips);
          console.log('转换后的灯带数量:', convertedStrips.length);

          setLedStrips(convertedStrips);

          if (convertedStrips.length > 0) {
            setSelectedStrip(convertedStrips[0]);
            console.log('设置默认选中的灯带:', convertedStrips[0].id);
          }

          console.log('✅ 成功加载已保存的LED灯带配置');

          // 配置已加载，createEffect 会自动启动单屏配置模式
          console.log('=== 配置已加载，等待 createEffect 自动启动单屏配置模式 ===');

          return; // 成功加载，不需要使用测试数据
        } else {
          console.log('No saved configuration found, starting with empty configuration');
        }
    } catch (error) {
      console.log('Failed to load saved configuration, starting with empty configuration:', error);
    }

    // 如果没有保存的配置或加载失败，从空配置开始
    console.log('No saved configuration found, starting with empty configuration');

    // 通过API命令报告状态，这样会显示在后端日志中
    try {
      await adaptiveApi.reportCurrentPage('🔧 单屏配置页面：从空配置开始');
    } catch (e) {
      console.error('Failed to report page info:', e);
    }

    // 设置空的灯带配置，用户需要手动添加
    setLedStrips([]);
    setSelectedStrip(null);

    console.log('=== 空配置已设置，用户可以手动添加LED灯带 ===');
  });

  // 组件卸载时的清理
  onCleanup(() => {
    console.log('🧹 SingleDisplayConfig 组件卸载，停止单屏配置模式');

    // 清理防抖定时器
    if (configModeRestartTimer) {
      clearTimeout(configModeRestartTimer);
      configModeRestartTimer = undefined;
    }

    // 停止所有LED效果
    const ledColorService = LedColorService.getInstance();
    ledStrips().forEach((strip) => {
      ledColorService.stopBreathingEffect(strip.id);
    });

    // 停止单屏配置模式
    stopSingleDisplayConfigMode();
  });

  // 创建新LED灯带
  const createLedStrip = async (border: 'Top' | 'Bottom' | 'Left' | 'Right') => {
    const isLongSide = border === 'Top' || border === 'Bottom';
    const defaultCount = isLongSide ? DEFAULT_CONFIG.longSide : DEFAULT_CONFIG.shortSide;

    // 获取下一个序号
    const existingSequences = ledStrips()
      .filter(s => s.driver === DEFAULT_CONFIG.driver)
      .map(s => s.sequence);
    const nextSequence = existingSequences.length > 0 ? Math.max(...existingSequences) + 1 : 1;

    const newStrip: LedStripConfig = {
      id: `strip_${Date.now()}_${Math.random()}`,
      displayId: displayId(),
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

    // 自动保存配置
    console.log('🔄 创建灯带后自动保存配置...');
    await autoSaveConfiguration();
  };

  // 更新LED灯带
  const updateLedStrip = async (updatedStrip: LedStripConfig) => {
    setLedStrips(prev => prev.map(strip =>
      strip.id === updatedStrip.id ? updatedStrip : strip
    ));
    setSelectedStrip(updatedStrip);

    // 自动保存配置
    console.log('🔄 更新灯带后自动保存配置...');
    await autoSaveConfiguration();
  };

  // 删除LED灯带
  const deleteLedStrip = async (stripId: string) => {
    setLedStrips(prev => prev.filter(strip => strip.id !== stripId));
    setSelectedStrip(null);

    // 自动保存配置
    console.log('🔄 删除灯带后自动保存配置...');
    await autoSaveConfiguration();
  };

  // 清空所有配置
  const clearAllConfig = async () => {
    setLedStrips([]);
    setSelectedStrip(null);

    // 自动保存配置（清空后的空配置）
    console.log('🔄 清空配置后自动保存...');
    await autoSaveConfiguration();
  };











  // 启动后端单屏配置模式
  const startSingleDisplayConfigMode = async () => {
    try {
      console.log('🚀 startSingleDisplayConfigMode 函数被调用');
      const currentStrips = ledStrips();
      console.log('🔍 当前灯带数量:', currentStrips.length);
      console.log('🔍 当前灯带详情:', currentStrips);

      if (currentStrips.length === 0) {
        console.log('⚠️ 没有配置的灯带，无法启动单屏配置模式');
        return;
      }

      // 转换为后端格式 - 匹配LedStripConfig结构
      const backendStrips = currentStrips.map(strip => ({
        index: strip.sequence, // 直接使用配置文件中的index值，不需要减1
        border: strip.border,
        display_id: strip.displayId,
        len: strip.count,
        led_type: strip.ledType,
        reversed: strip.reverse, // 注意：后端字段名是reversed，不是reverse
      }));

      // 定义边框颜色 - 与ColorPreview组件和后端测试代码一致
      const borderColors = {
        top: [[0, 255, 255], [0, 0, 255]],       // 青色 (180°) + 蓝色 (225°)
        bottom: [[255, 0, 0], [255, 128, 0]],    // 红色 (0°) + 橙色 (45°)
        left: [[128, 0, 255], [255, 0, 128]],    // 紫色 (270°) + 玫红色 (315°)
        right: [[255, 255, 0], [128, 255, 0]],   // 黄色 (90°) + 黄绿色 (135°)
      };

      console.log('=== 启动后端单屏配置模式 ===');
      console.log('灯带配置:', backendStrips);
      console.log('边框颜色:', borderColors);

      await adaptiveApi.startSingleDisplayConfigPublisher(backendStrips, borderColors);

      console.log('✅ 后端单屏配置模式已启动');
    } catch (error) {
      console.error('❌ 启动后端单屏配置模式失败:', error);
    }
  };

  // 停止后端单屏配置模式
  const stopSingleDisplayConfigMode = async () => {
    try {
      console.log('=== 停止后端单屏配置模式 ===');
      await adaptiveApi.stopSingleDisplayConfigPublisher();
      console.log('✅ 后端单屏配置模式已停止');
    } catch (error) {
      console.error('❌ 停止后端单屏配置模式失败:', error);
    }
  };

  // 自动保存配置到后端
  const autoSaveConfiguration = async () => {
    try {
      console.log('🔄 自动保存配置到后端（V2 读改写）...');
      const currentStrips = ledStrips();
      console.log('🔄 当前灯带数量:', currentStrips.length);

      // 获取 internal_id 映射
      const displayConfigs = await adaptiveApi.getDisplayConfigs();
      const currentDisplayId = displayId();
      const currentDisplay = Array.isArray(displayConfigs)
        ? displayConfigs.find((d: any) => d.last_system_id === currentDisplayId)
        : undefined;
      const targetInternalId = currentDisplay ? currentDisplay.internal_id : undefined;
      if (!targetInternalId) {
        console.warn('⚠️ 未找到当前显示器的 internal_id，跳过保存');
        return;
      }

      // 读取现有 V2 配置组
      const v2Group = await adaptiveApi.readLedStripConfigs();

      // 将当前 UI 条目转换为 V2 条目（绑定到当前 internal_id）
      const backendStripsV2 = currentStrips.map(strip => ({
        index: strip.sequence,
        border: strip.border,
        display_internal_id: targetInternalId,
        len: strip.count,
        led_type: strip.ledType,
        reversed: strip.reverse,
      }));

      // 保留其它显示器的条目，仅替换当前 internal_id 的条目
      const existingStrips: any[] = Array.isArray((v2Group as any).strips) ? (v2Group as any).strips : [];
      const otherStrips = existingStrips.filter((s: any) => s.display_internal_id !== targetInternalId);
      (v2Group as any).strips = [...otherStrips, ...backendStripsV2];

      console.log('🔄 自动保存的配置数据（V2 完整组）:', v2Group);

      // 写回 V2 配置组
      await adaptiveApi.writeLedStripConfigs(v2Group);

      console.log('✅ 配置自动保存成功！（V2）');

    } catch (error) {
      console.error('❌ 自动保存配置失败:', error);
      // 自动保存失败时不显示弹窗，只记录日志
    }
  };

  // 调试函数：显示当前配置信息
  const debugCurrentConfig = () => {
    console.log('🚀 debugCurrentConfig 函数被调用！');
    console.log('🚀 这是调试函数执行的第一行日志');

    try {
      const currentStrips = ledStrips();
      console.log('🔍 当前LED灯带配置调试信息:');
      console.log(`总灯带数量: ${currentStrips.length}`);

    if (currentStrips.length === 0) {
      console.log('⚠️ 没有找到任何LED灯带配置');
      return;
    }

    const sortedStrips = [...currentStrips].sort((a, b) => a.sequence - b.sequence);
    console.log('📋 灯带详细信息:');

    let cumulativeLedOffset = 0;
    if (import.meta.env.DEV) {
      sortedStrips.forEach((strip, index) => {
        const bytesPerLed = strip.ledType === 'SK6812' ? 4 : 3;
        const byteOffset = cumulativeLedOffset * bytesPerLed;

        console.log(`${index + 1}. 灯带 ${strip.id}:`);
        console.log(`   - 边框: ${strip.border}, LED数量: ${strip.count}, 类型: ${strip.ledType}`);
        console.log(`   - 累积LED偏移: ${cumulativeLedOffset}, 字节偏移: ${byteOffset}`);

        cumulativeLedOffset += strip.count;
      });
    } else {
      // 在生产模式下只计算偏移量，不打印日志
      sortedStrips.forEach((strip) => {
        cumulativeLedOffset += strip.count;
      });
    }

    // 检查序列号重复
    const sequences = sortedStrips.map(s => s.sequence);
    const duplicates = sequences.filter((seq, index) => sequences.indexOf(seq) !== index);
    if (duplicates.length > 0) {
      console.error(`❌ 发现重复的序列号: ${[...new Set(duplicates)].join(', ')}`);
    } else {
      console.log('✅ 所有序列号都是唯一的');
    }

    console.log(`📊 总计: ${cumulativeLedOffset} 个LED`);

    alert(`调试信息已输出到控制台。当前有 ${currentStrips.length} 个灯带配置，总计 ${cumulativeLedOffset} 个LED。`);
    } catch (error) {
      console.error('❌ 调试函数执行失败:', error);
      alert('❌ 调试函数执行失败: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  // 设置活跃灯带用于呼吸效果
  const setActiveStripForBreathing = async (strip: LedStripConfig | null) => {
    try {
      if (strip) {
        console.log('设置活跃灯带用于呼吸效果:', strip.id, strip.border);
        await adaptiveApi.setActiveStripForBreathing(strip.displayId, strip.border);
      } else {
        console.log('清除活跃灯带呼吸效果');
        await adaptiveApi.setActiveStripForBreathing(displayId(), null);
      }
    } catch (error) {
      console.error('设置活跃灯带失败:', error);
    }
  };

  // 监听选中和悬浮状态变化，设置活跃灯带
  createEffect(() => {
    const selected = selectedStrip();
    const hovered = hoveredStrip();

    // 悬浮优先，只能有一个是活动状态
    const activeStrip = hovered || selected;

    console.log('活跃灯带状态变化:', {
      selected: selected?.id || 'none',
      hovered: hovered?.id || 'none',
      active: activeStrip?.id || 'none'
    });

    setActiveStripForBreathing(activeStrip);
  });

  // 防抖的单屏配置模式启动
  let configModeRestartTimer: ReturnType<typeof setTimeout> | undefined;

  // 当灯带配置变化时，防抖重新启动后端单屏配置模式
  createEffect(() => {
    const strips = ledStrips();

    // 只监听关键配置变化，避免过度触发
    const stripSignature = strips.map(strip =>
      `${strip.id}-${strip.count}-${strip.reverse}-${strip.ledType}`
    ).join('|');

    // 清除之前的定时器
    if (configModeRestartTimer) {
      clearTimeout(configModeRestartTimer);
    }

    if (strips.length > 0) {
      console.log(`=== 检测到${strips.length}个已配置的灯带，准备启动后端单屏配置模式 ===`);
      console.log(`配置签名: ${stripSignature}`);

      // 使用较短的防抖延迟，快速响应配置变化
      configModeRestartTimer = setTimeout(() => {
        console.log('🚀 防抖延迟后启动单屏配置模式');
        startSingleDisplayConfigMode();
      }, 300); // 减少到300ms防抖延迟
    } else {
      console.log('=== 没有配置的灯带，停止后端单屏配置模式 ===');
      stopSingleDisplayConfigMode();
    }
  });



  return (
    <div class="container mx-auto p-6 h-full">
      {/* WebSocket监听器 */}
      <WebSocketListener />

      <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">{t('singleDisplayConfig.title')}</h1>
        <div class="flex gap-2 items-center">
          <button
            class="btn btn-outline btn-info"
            on:click={debugCurrentConfig}
            title="在控制台显示调试信息"
          >
            调试信息
          </button>
          <button
            class="btn btn-outline btn-error"
            on:click={async () => await clearAllConfig()}
          >
            {t('common.clear')}
          </button>
          <button
            class="btn btn-outline"
            on:click={() => navigate('/led-strips-configuration')}
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
                      hoveredStrip={hoveredStrip()}
                      onHoverStrip={(strip) => {
                        console.log('Setting hovered strip:', strip?.id || 'null');
                        setHoveredStrip(strip);
                      }}
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
        <div class="lg:col-span-1 space-y-4">
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
