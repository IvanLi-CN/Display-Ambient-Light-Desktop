import { Component, createSignal, For, createEffect, onCleanup } from 'solid-js';
import { createStore } from 'solid-js/store';
import { useLanguage } from '../../i18n/index';
import { Borders } from '../../constants/border';
import { LedType } from '../../models/led-strip-config';
import { LedColorService } from '../../services/led-color-service';

// 虚拟灯带配置
export interface VirtualLedStrip {
  id: string;
  border: Borders;
  count: number;
  ledType: LedType;
  reversed: boolean;
  driverId: number;
  stripOrder: number;
  startOffset: number; // 0-100 百分比
  endOffset: number;   // 0-100 百分比
}

// 默认LED配置
const DEFAULT_LED_CONFIG = {
  topCount: 38,
  bottomCount: 38,
  leftCount: 22,
  rightCount: 22,
  ledType: LedType.WS2812B,
};

type VirtualDisplayProps = {
  displayId: number;
  onStripSelect?: (strip: VirtualLedStrip | null) => void;
  onStripHover?: (strip: VirtualLedStrip | null) => void;
  onStripUpdate?: (stripId: string, updates: Partial<VirtualLedStrip>) => void;
  onStripDelete?: (stripId: string) => void;
  onStripCreate?: (strip: VirtualLedStrip) => void;
  selectedStrip?: VirtualLedStrip | null;
  hoveredStrip?: VirtualLedStrip | null;
  strips?: VirtualLedStrip[];
};

export const VirtualDisplay: Component<VirtualDisplayProps> = (props) => {
  const { t } = useLanguage();

  // 灯带配置存储 - 如果外部提供了strips则使用外部的，否则使用内部状态
  const [internalStrips, setInternalStrips] = createStore<VirtualLedStrip[]>([]);
  const strips = () => props.strips || internalStrips;

  // LED颜色服务
  const colorService = LedColorService.getInstance();
  
  // 生成唯一ID
  const generateId = () => `strip_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

  // 清理颜色效果
  onCleanup(() => {
    colorService.cleanup();
  });

  // 更新灯带颜色效果 (仅用于UI展示，不发送到硬件)
  const updateStripColors = async (strip: VirtualLedStrip) => {
    const isSelected = props.selectedStrip?.id === strip.id;
    const isHovered = props.hoveredStrip?.id === strip.id;

    // 注意：VirtualDisplay组件主要用于UI展示，不直接发送数据到硬件
    // 实际的硬件数据发送由父组件处理
    console.log('Virtual display strip color update:', {
      stripId: strip.id,
      border: strip.border,
      isSelected,
      isHovered
    });
  };

  // 监听选中状态变化
  createEffect(() => {
    const selectedStrip = props.selectedStrip;
    if (selectedStrip) {
      updateStripColors(selectedStrip);
    }
  });

  // 监听悬浮状态变化
  createEffect(() => {
    const hoveredStrip = props.hoveredStrip;
    if (hoveredStrip) {
      updateStripColors(hoveredStrip);
    }
  });

  // 创建灯带
  const createStrip = (border: Borders) => {
    const defaultCount = border === 'Top' || border === 'Bottom'
      ? DEFAULT_LED_CONFIG.topCount
      : DEFAULT_LED_CONFIG.leftCount;

    // 找到该驱动器的最大序号
    const currentStrips = strips();
    const existingOrders = currentStrips.filter(s => s.driverId === 1).map(s => s.stripOrder);
    const maxOrder = existingOrders.length > 0 ? Math.max(...existingOrders) : 0;

    const newStrip: VirtualLedStrip = {
      id: generateId(),
      border,
      count: defaultCount,
      ledType: DEFAULT_LED_CONFIG.ledType,
      reversed: false,
      driverId: 1, // 默认驱动器1
      stripOrder: maxOrder + 1,
      startOffset: 0,
      endOffset: 100,
    };

    if (props.onStripCreate) {
      props.onStripCreate(newStrip);
    } else {
      setInternalStrips(prev => [...prev, newStrip]);
    }

    // 立即更新新灯带的颜色
    setTimeout(() => updateStripColors(newStrip), 100);

    props.onStripSelect?.(newStrip);
  };

  // 删除灯带
  const deleteStrip = (stripId: string) => {
    // 停止颜色效果
    colorService.stopBreathingEffect(stripId);

    if (props.onStripDelete) {
      props.onStripDelete(stripId);
    } else {
      setInternalStrips(prev => prev.filter(s => s.id !== stripId));
    }
    if (props.selectedStrip?.id === stripId) {
      props.onStripSelect?.(null);
    }
  };

  // 更新灯带
  const updateStrip = (stripId: string, updates: Partial<VirtualLedStrip>) => {
    if (props.onStripUpdate) {
      props.onStripUpdate(stripId, updates);
    } else {
      setInternalStrips((prev) => prev.map(s => s.id === stripId ? { ...s, ...updates } : s));
    }

    // 更新后重新设置颜色
    const updatedStrip = strips().find(s => s.id === stripId);
    if (updatedStrip) {
      setTimeout(() => updateStripColors({ ...updatedStrip, ...updates }), 100);
    }
  };

  // 获取边框的灯带
  const getStripsForBorder = (border: Borders) => {
    return strips().filter(s => s.border === border);
  };

  // 渲染边框槽位
  const renderBorderSlots = (border: Borders, className: string) => {
    const borderStrips = getStripsForBorder(border);
    const isHorizontal = border === 'Top' || border === 'Bottom';
    
    return (
      <div class={`${className} flex ${isHorizontal ? 'flex-row' : 'flex-col'} gap-1 p-2`}>
        {/* 现有灯带 */}
        <For each={borderStrips}>
          {(strip) => (
            <div
              class={`
                border-2 border-dashed border-primary/50 rounded p-2 cursor-pointer
                hover:border-primary hover:bg-primary/10 transition-all
                ${props.selectedStrip?.id === strip.id ? 'border-primary bg-primary/20' : ''}
                ${props.hoveredStrip?.id === strip.id ? 'border-primary/80 bg-primary/15' : ''}
                ${isHorizontal ? 'min-w-[60px]' : 'min-h-[60px]'}
              `}
              onClick={() => props.onStripSelect?.(strip)}
              onMouseEnter={() => props.onStripHover?.(strip)}
              onMouseLeave={() => props.onStripHover?.(null)}
            >
              <div class="text-xs text-center">
                <div class="font-semibold">{strip.count} LEDs</div>
                <div class="text-primary">{strip.ledType}</div>
                <div class="text-xs opacity-70">#{strip.stripOrder}</div>
              </div>
            </div>
          )}
        </For>
        
        {/* 添加新灯带按钮 */}
        <div
          class={`
            border-2 border-dashed border-base-300 rounded p-2 cursor-pointer
            hover:border-primary hover:bg-primary/5 transition-all
            flex items-center justify-center
            ${isHorizontal ? 'min-w-[60px] h-16' : 'min-h-[60px] w-16'}
          `}
          onClick={() => createStrip(border)}
        >
          <div class="text-center">
            <div class="text-2xl text-base-content/50">+</div>
            <div class="text-xs text-base-content/70">{t('common.add')}</div>
          </div>
        </div>
      </div>
    );
  };

  // 渲染假显示器中央的颜色指示
  const renderColorIndicator = () => {
    return (
      <div class="w-full h-full bg-gradient-to-r from-red-500 via-green-500 to-blue-500 rounded flex items-center justify-center">
        <div class="bg-black/50 text-white px-4 py-2 rounded text-center">
          <div class="text-lg font-semibold">{t('singleDisplayConfig.colorIndicator')}</div>
          <div class="text-sm opacity-80">{t('singleDisplayConfig.colorIndicatorDesc')}</div>
        </div>
      </div>
    );
  };

  return (
    <div class="flex flex-col items-center gap-4 p-8">
      {/* 虚拟显示器网格布局 */}
      <div class="grid grid-cols-[200px,400px,200px] grid-rows-[100px,300px,100px] gap-2">
        {/* 顶部槽位 */}
        <div class="col-start-2 row-start-1">
          {renderBorderSlots('Top', 'h-full')}
        </div>
        
        {/* 左侧槽位 */}
        <div class="col-start-1 row-start-2">
          {renderBorderSlots('Left', 'h-full')}
        </div>
        
        {/* 中央假显示器 */}
        <div class="col-start-2 row-start-2 border-4 border-base-300 rounded-lg overflow-hidden">
          {renderColorIndicator()}
        </div>
        
        {/* 右侧槽位 */}
        <div class="col-start-3 row-start-2">
          {renderBorderSlots('Right', 'h-full')}
        </div>
        
        {/* 底部槽位 */}
        <div class="col-start-2 row-start-3">
          {renderBorderSlots('Bottom', 'h-full')}
        </div>
      </div>
      
      {/* 说明文字 */}
      <div class="text-center text-sm text-base-content/70 max-w-2xl">
        <p>{t('singleDisplayConfig.virtualDisplayInstructions')}</p>
      </div>
    </div>
  );
};

// 导出类型和组件
export { VirtualDisplay as default };
