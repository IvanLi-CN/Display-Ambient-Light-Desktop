#!/usr/bin/env python3
"""
创建测试LED配置文件
"""

import os
from pathlib import Path

def create_test_led_config():
    """创建测试LED配置文件"""
    print("📦 创建测试LED配置文件...")
    
    # 配置文件路径
    config_dir = Path.home() / "Library" / "Application Support" / "cc.ivanli.ambient_light"
    config_file = config_dir / "led_strip_config.toml"
    
    print(f"📁 配置文件路径: {config_file}")
    
    # 创建配置目录
    config_dir.mkdir(parents=True, exist_ok=True)
    
    # 创建TOML格式的配置内容
    toml_content = '''# LED Strip Configuration for Display Ambient Light Control

[[strips]]
index = 0
border = "Bottom"
display_id = 2
start_pos = 0
len = 38
led_type = "SK6812"

[[strips]]
index = 1
border = "Right"
display_id = 2
start_pos = 152
len = 22
led_type = "WS2812B"

[[strips]]
index = 2
border = "Top"
display_id = 2
start_pos = 218
len = 38
led_type = "SK6812"

[[strips]]
index = 3
border = "Left"
display_id = 2
start_pos = 370
len = 22
led_type = "WS2812B"

[[mappers]]
start = 0
end = 38
pos = 0

[[mappers]]
start = 38
end = 60
pos = 152

[[mappers]]
start = 60
end = 98
pos = 218

[[mappers]]
start = 98
end = 120
pos = 370

[color_calibration]
r = 1.0
g = 1.0
b = 1.0
w = 1.0
'''

    # 写入配置文件
    try:
        with open(config_file, 'w') as f:
            f.write(toml_content)
        
        print("✅ 测试LED配置文件创建成功")
        print(f"📄 文件位置: {config_file}")
        print()
        print("📊 配置内容:")
        print("   Bottom: 38 LEDs (SK6812) at offset 0")
        print("   Right: 22 LEDs (WS2812B) at offset 152")
        print("   Top: 38 LEDs (SK6812) at offset 218")
        print("   Left: 22 LEDs (WS2812B) at offset 370")
        
        return True
        
    except Exception as e:
        print(f"❌ 创建配置文件失败: {e}")
        return False

def verify_config_file():
    """验证配置文件"""
    config_dir = Path.home() / "Library" / "Application Support" / "cc.ivanli.ambient_light"
    config_file = config_dir / "led_strip_config.toml"
    
    print(f"\n🔍 验证配置文件: {config_file}")
    
    if config_file.exists():
        try:
            with open(config_file, 'r') as f:
                content = f.read()

            # 简单检查TOML文件是否包含预期的内容
            if "[[strips]]" in content and "border" in content:
                print("✅ 配置文件存在且格式正确")
                strip_count = content.count("[[strips]]")
                print(f"📊 包含 {strip_count} 个LED灯带配置")
                return True
            else:
                print("❌ 配置文件格式不正确")
                return False

        except Exception as e:
            print(f"❌ 配置文件读取错误: {e}")
            return False
    else:
        print("❌ 配置文件不存在")
        return False

def main():
    """主函数"""
    print("🔧 LED配置文件创建工具")
    print("为LED配置界面创建测试数据")
    print()
    
    # 检查现有配置文件
    if verify_config_file():
        choice = input("配置文件已存在，是否覆盖? (y/N): ").strip().lower()
        if choice != 'y':
            print("取消操作")
            return
    
    # 创建测试配置文件
    if create_test_led_config():
        print()
        print("🎉 配置文件创建完成！")
        print()
        print("📋 下一步操作:")
        print("1. 重新启动桌面应用程序（或重新加载配置）")
        print("2. 在浏览器中访问: http://localhost:1420/led-strips-configuration/display/2")
        print("3. 检查LED配置界面是否开始发送测试数据")
        print("4. 观察虚拟驱动板是否接收到数据")
    else:
        print("❌ 配置文件创建失败")

if __name__ == "__main__":
    main()
