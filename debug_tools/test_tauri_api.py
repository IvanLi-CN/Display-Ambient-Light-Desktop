#!/usr/bin/env python3
"""
测试Tauri API调用 - 直接调用后端API来诊断问题
"""

import json
import time

def test_tauri_api():
    """测试Tauri API调用"""
    base_url = "http://localhost:1420"
    
    print("🔧 Tauri API测试工具")
    print("直接调用后端API来诊断LED配置问题")
    print()
    
    # 测试API调用
    test_cases = [
        {
            "name": "获取硬件设备列表",
            "method": "get_boards",
            "params": {}
        },
        {
            "name": "启用测试模式", 
            "method": "enable_test_mode",
            "params": {}
        },
        {
            "name": "读取LED配置",
            "method": "read_led_strip_configs", 
            "params": {}
        },
        {
            "name": "获取显示器信息",
            "method": "list_display_info",
            "params": {}
        }
    ]
    
    for test_case in test_cases:
        print(f"🧪 测试: {test_case['name']}")
        try:
            # 构造API调用
            payload = {
                "cmd": test_case["method"],
                **test_case["params"]
            }
            
            # 发送请求（注意：这种方法可能不适用于Tauri应用）
            # Tauri应用通常不提供HTTP API，而是通过IPC通信
            print(f"   ⚠️ 注意：Tauri应用不提供HTTP API，需要通过前端调用")
            print(f"   📋 应该调用的方法: {test_case['method']}")
            print(f"   📋 参数: {test_case['params']}")
            
        except Exception as e:
            print(f"   ❌ 测试失败: {e}")
        
        print()

def create_test_led_config():
    """创建测试LED配置数据"""
    print("📦 创建测试LED配置数据...")
    
    # 模拟LED配置数据（后端格式）
    test_config = [
        {
            "index": 0,
            "border": "Bottom",
            "display_id": 2,
            "start_pos": 0,
            "len": 38,
            "led_type": "SK6812"
        },
        {
            "index": 1,
            "border": "Right", 
            "display_id": 2,
            "start_pos": 152,  # 38 * 4 = 152
            "len": 22,
            "led_type": "WS2812B"
        },
        {
            "index": 2,
            "border": "Top",
            "display_id": 2, 
            "start_pos": 218,  # 152 + 22 * 3 = 218
            "len": 38,
            "led_type": "SK6812"
        },
        {
            "index": 3,
            "border": "Left",
            "display_id": 2,
            "start_pos": 370,  # 218 + 38 * 4 = 370
            "len": 22,
            "led_type": "WS2812B"
        }
    ]
    
    print("✅ 测试LED配置数据:")
    for config in test_config:
        print(f"   {config['border']}: {config['len']} LEDs ({config['led_type']}) at offset {config['start_pos']}")
    
    return test_config

def analyze_led_config_problem():
    """分析LED配置问题"""
    print("🔍 LED配置问题分析")
    print()
    
    print("📋 可能的问题原因:")
    print("1. LED配置界面没有被正确访问")
    print("   - URL应该是: http://localhost:1420/led-strips-configuration/display/2")
    print("   - 检查浏览器是否成功导航到该页面")
    print()
    
    print("2. 没有LED配置数据")
    print("   - LED配置界面需要有配置数据才会启动测试发送")
    print("   - 检查是否有保存的LED配置文件")
    print()
    
    print("3. 硬件设备检测失败")
    print("   - get_boards() 可能返回空列表")
    print("   - 检查硬件设备是否被正确检测")
    print()
    
    print("4. 测试模式没有启用")
    print("   - enable_test_mode() 可能没有被调用")
    print("   - 检查测试模式状态")
    print()
    
    print("5. 前端代码逻辑问题")
    print("   - startTestColorSending() 可能没有被触发")
    print("   - 检查前端控制台是否有错误信息")
    print()

def suggest_debugging_steps():
    """建议调试步骤"""
    print("🛠️ 建议的调试步骤:")
    print()
    
    print("1. 检查前端访问:")
    print("   - 在浏览器中访问: http://localhost:1420/led-strips-configuration/display/2")
    print("   - 打开浏览器开发者工具，查看控制台输出")
    print("   - 确认页面是否正确加载")
    print()
    
    print("2. 检查API调用:")
    print("   - 在浏览器控制台中手动调用:")
    print("   - window.__TAURI__.core.invoke('get_boards')")
    print("   - window.__TAURI__.core.invoke('enable_test_mode')")
    print("   - window.__TAURI__.core.invoke('read_led_strip_configs')")
    print()
    
    print("3. 手动触发测试数据发送:")
    print("   - 在浏览器控制台中调用:")
    print("   - window.__TAURI__.core.invoke('send_test_colors_to_board', {")
    print("       boardAddress: '<BOARD_IP>:<BOARD_PORT>',")
    print("       offset: 0,")
    print("       buffer: [255, 0, 0, 0, 255, 0, 0, 0, 255]")
    print("     })")
    print()
    
    print("4. 创建测试LED配置:")
    print("   - 手动创建LED配置数据")
    print("   - 保存到配置文件")
    print("   - 重新加载LED配置界面")
    print()

def main():
    """主函数"""
    test_tauri_api()
    create_test_led_config()
    analyze_led_config_problem()
    suggest_debugging_steps()
    
    print("💡 下一步建议:")
    print("1. 在浏览器中访问LED配置界面")
    print("2. 打开开发者工具查看控制台输出")
    print("3. 手动调用API测试功能")
    print("4. 如果需要，创建测试LED配置数据")

if __name__ == "__main__":
    main()
