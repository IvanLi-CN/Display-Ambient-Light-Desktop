import { adaptiveApi } from './api-adapter';
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

  // HSV到RGB转换函数
  private hsvToRgb(h: number, s: number, v: number): { r: number; g: number; b: number } {
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
  }

  // 生成边框颜色（左下角为原点，顺时针）- 使用色环每45度的颜色
  public generateBorderColors(border: string, ledCount: number): LedColorData[] {
    const colors: LedColorData[] = [];

    // 每个边被两个颜色平分
    const halfCount = Math.floor(ledCount / 2);

    // 色环每45度的颜色定义 (HSV: H=色相, S=1.0, V=1.0)
    const colorWheel45Degrees = [
      this.hsvToRgb(0, 1.0, 1.0),    // 0° - 红色
      this.hsvToRgb(45, 1.0, 1.0),   // 45° - 橙色
      this.hsvToRgb(90, 1.0, 1.0),   // 90° - 黄色
      this.hsvToRgb(135, 1.0, 1.0),  // 135° - 黄绿色
      this.hsvToRgb(180, 1.0, 1.0),  // 180° - 青色
      this.hsvToRgb(225, 1.0, 1.0),  // 225° - 蓝色
      this.hsvToRgb(270, 1.0, 1.0),  // 270° - 紫色
      this.hsvToRgb(315, 1.0, 1.0),  // 315° - 玫红色
    ];

    switch (border.toLowerCase()) {
      case 'bottom':
        // 底边：左半部分红色(0°)，右半部分橙色(45°)
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            const color = colorWheel45Degrees[0]; // 红色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          } else {
            const color = colorWheel45Degrees[1]; // 橙色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          }
        }
        break;
      case 'right':
        // 右边：下半部分黄色(90°)，上半部分黄绿色(135°)
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            const color = colorWheel45Degrees[2]; // 黄色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          } else {
            const color = colorWheel45Degrees[3]; // 黄绿色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          }
        }
        break;
      case 'top':
        // 顶边：右半部分青色(180°)，左半部分蓝色(225°)
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            const color = colorWheel45Degrees[4]; // 青色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          } else {
            const color = colorWheel45Degrees[5]; // 蓝色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          }
        }
        break;
      case 'left':
        // 左边：上半部分紫色(270°)，下半部分玫红色(315°)
        for (let i = 0; i < ledCount; i++) {
          if (i < halfCount) {
            const color = colorWheel45Degrees[6]; // 紫色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          } else {
            const color = colorWheel45Degrees[7]; // 玫红色
            colors.push({ r: color.r, g: color.g, b: color.b, w: 0 }); // 白色通道设为0
          }
        }
        break;
      default:
        // 默认白色 - 对于SK6812不点亮白色通道
        for (let i = 0; i < ledCount; i++) {
          colors.push({ r: 255, g: 255, b: 255, w: 0 }); // 白色通道设为0
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
    boardAddress: string
  ): Promise<void> {
    try {
      const colorBytes = this.colorsToBytes(colors, strip.ledType);
      const bytesPerLed = strip.ledType === LedType.SK6812 ? 4 : 3;
      const offset = strip.stripOrder * strip.count * bytesPerLed;
      
      await adaptiveApi.sendTestColorsToBoard({
        boardAddress,
        offset,
        buffer: Array.from(colorBytes)
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
    boardAddress: string
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
