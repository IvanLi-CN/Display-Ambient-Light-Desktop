import { invoke } from '@tauri-apps/api/core';
import { VirtualLedStrip } from '../components/led-strip-configuration/virtual-display';
import { LedType } from '../models/led-strip-config';

// LED颜色数据结构
export interface LedColorData {
  r: number;
  g: number;
  b: number;
  w?: number; // 白色通道，仅RGBW使用
}

// 呼吸效果参数
export interface BreathingEffect {
  enabled: boolean;
  speed: number; // 呼吸速度 (0.1 - 2.0)
  minBrightness: number; // 最小亮度 (0-255)
  maxBrightness: number; // 最大亮度 (0-255)
}

// LED颜色服务类
export class LedColorService {
  private static instance: LedColorService;
  private breathingAnimations: Map<string, number> = new Map(); // 存储动画ID
  private currentColors: Map<string, LedColorData[]> = new Map(); // 存储当前颜色

  private constructor() {}

  public static getInstance(): LedColorService {
    if (!LedColorService.instance) {
      LedColorService.instance = new LedColorService();
    }
    return LedColorService.instance;
  }

  // 生成边框颜色（左下角为原点，顺时针）
  public generateBorderColors(border: string, ledCount: number): LedColorData[] {
    const colors: LedColorData[] = [];
    
    // 每个边被两个颜色平分
    const halfCount = Math.floor(ledCount / 2);
    
    switch (border.toLowerCase()) {
      case 'bottom':
        // 底边：左半部分红色，右半部分绿色
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            colors.push({ r: 255, g: 0, b: 0 }); // 红色
          } else {
            colors.push({ r: 0, g: 255, b: 0 }); // 绿色
          }
        }
        break;
      case 'right':
        // 右边：下半部分绿色，上半部分蓝色
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            colors.push({ r: 0, g: 255, b: 0 }); // 绿色
          } else {
            colors.push({ r: 0, g: 0, b: 255 }); // 蓝色
          }
        }
        break;
      case 'top':
        // 顶边：右半部分蓝色，左半部分黄色
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            colors.push({ r: 0, g: 0, b: 255 }); // 蓝色
          } else {
            colors.push({ r: 255, g: 255, b: 0 }); // 黄色
          }
        }
        break;
      case 'left':
        // 左边：上半部分黄色，下半部分红色
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            colors.push({ r: 255, g: 255, b: 0 }); // 黄色
          } else {
            colors.push({ r: 255, g: 0, b: 0 }); // 红色
          }
        }
        break;
      default:
        // 默认白色
        for (let i = 0; i < ledCount; i++) {
          colors.push({ r: 255, g: 255, b: 255 });
        }
    }
    
    return colors;
  }

  // 应用亮度到颜色
  public applyBrightness(colors: LedColorData[], brightness: number): LedColorData[] {
    const factor = Math.max(0, Math.min(255, brightness)) / 255;
    return colors.map(color => ({
      r: Math.round(color.r * factor),
      g: Math.round(color.g * factor),
      b: Math.round(color.b * factor),
      w: color.w ? Math.round(color.w * factor) : undefined,
    }));
  }

  // 启动呼吸效果
  public startBreathingEffect(stripId: string, baseColors: LedColorData[], effect: BreathingEffect): void {
    this.stopBreathingEffect(stripId); // 停止现有动画
    
    if (!effect.enabled) return;
    
    let startTime = Date.now();
    const animate = () => {
      const elapsed = (Date.now() - startTime) / 1000; // 秒
      const cycle = (elapsed * effect.speed) % (Math.PI * 2); // 呼吸周期
      const breathFactor = (Math.sin(cycle) + 1) / 2; // 0-1之间的呼吸因子
      
      // 计算当前亮度
      const currentBrightness = effect.minBrightness + 
        (effect.maxBrightness - effect.minBrightness) * breathFactor;
      
      // 应用亮度并发送颜色
      const breathingColors = this.applyBrightness(baseColors, currentBrightness);
      this.currentColors.set(stripId, breathingColors);
      
      // 继续动画
      const animationId = requestAnimationFrame(animate);
      this.breathingAnimations.set(stripId, animationId);
    };
    
    animate();
  }

  // 停止呼吸效果
  public stopBreathingEffect(stripId: string): void {
    const animationId = this.breathingAnimations.get(stripId);
    if (animationId) {
      cancelAnimationFrame(animationId);
      this.breathingAnimations.delete(stripId);
    }
  }

  // 设置静态颜色
  public setStaticColors(stripId: string, colors: LedColorData[], brightness: number = 255): void {
    this.stopBreathingEffect(stripId);
    const adjustedColors = this.applyBrightness(colors, brightness);
    this.currentColors.set(stripId, adjustedColors);
  }

  // 获取当前颜色
  public getCurrentColors(stripId: string): LedColorData[] | undefined {
    return this.currentColors.get(stripId);
  }

  // 将颜色数据转换为字节数组
  public colorsToBytes(colors: LedColorData[], ledType: LedType): Uint8Array {
    const bytesPerLed = ledType === LedType.SK6812 ? 4 : 3;
    const buffer = new Uint8Array(colors.length * bytesPerLed);
    
    for (let i = 0; i < colors.length; i++) {
      const color = colors[i];
      const offset = i * bytesPerLed;
      
      if (ledType === LedType.SK6812) {
        // SK6812: G,R,B,W 顺序
        buffer[offset] = color.g;
        buffer[offset + 1] = color.r;
        buffer[offset + 2] = color.b;
        buffer[offset + 3] = color.w || 0;
      } else {
        // WS2812B: G,R,B 顺序
        buffer[offset] = color.g;
        buffer[offset + 1] = color.r;
        buffer[offset + 2] = color.b;
      }
    }
    
    return buffer;
  }

  // 发送颜色到硬件
  public async sendColorsToHardware(
    strip: VirtualLedStrip, 
    colors: LedColorData[], 
    boardAddress: string = '192.168.4.1:8888'
  ): Promise<void> {
    try {
      const colorBytes = this.colorsToBytes(colors, strip.ledType);
      const bytesPerLed = strip.ledType === LedType.SK6812 ? 4 : 3;
      const offset = strip.stripOrder * strip.count * bytesPerLed;
      
      await invoke('send_test_colors_to_board', {
        boardAddress,
        offset,
        buffer: Array.from(colorBytes),
      });
    } catch (error) {
      console.error('Failed to send colors to hardware:', error);
      throw error;
    }
  }

  // 更新灯带颜色（用于配置界面）
  public async updateStripColors(
    strip: VirtualLedStrip,
    isSelected: boolean = false,
    isHovered: boolean = false,
    boardAddress: string = '192.168.4.1:8888'
  ): Promise<void> {
    // 生成基础颜色
    const baseColors = this.generateBorderColors(strip.border, strip.count);
    
    // 根据状态设置亮度和呼吸效果
    let brightness = 100; // 默认亮度
    let breathingEffect: BreathingEffect = {
      enabled: false,
      speed: 1.0,
      minBrightness: 50,
      maxBrightness: 200,
    };
    
    if (isSelected || isHovered) {
      brightness = isSelected ? 150 : 120;
      breathingEffect = {
        enabled: true,
        speed: 0.8,
        minBrightness: 50,
        maxBrightness: 200,
      };
    }
    
    if (breathingEffect.enabled) {
      this.startBreathingEffect(strip.id, baseColors, breathingEffect);
      
      // 启动发送循环
      const sendLoop = async () => {
        const currentColors = this.getCurrentColors(strip.id);
        if (currentColors) {
          try {
            await this.sendColorsToHardware(strip, currentColors, boardAddress);
          } catch (error) {
            console.error('Failed to send breathing colors:', error);
          }
        }
        
        // 如果呼吸效果仍在运行，继续发送
        if (this.breathingAnimations.has(strip.id)) {
          setTimeout(sendLoop, 50); // 20 FPS
        }
      };
      sendLoop();
    } else {
      this.setStaticColors(strip.id, baseColors, brightness);
      const currentColors = this.getCurrentColors(strip.id);
      if (currentColors) {
        await this.sendColorsToHardware(strip, currentColors, boardAddress);
      }
    }
  }

  // 清理所有效果
  public cleanup(): void {
    for (const stripId of this.breathingAnimations.keys()) {
      this.stopBreathingEffect(stripId);
    }
    this.currentColors.clear();
  }
}
