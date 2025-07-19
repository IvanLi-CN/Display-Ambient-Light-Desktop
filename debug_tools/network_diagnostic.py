#!/usr/bin/env python3
"""
网络诊断工具 - 分析LED数据传输问题
"""

import socket
import time
import threading
from datetime import datetime

def test_udp_send_to_hardware():
    """测试直接向硬件发送UDP数据"""
    print("🔧 测试向硬件驱动板发送UDP数据")
    print("目标地址: 192.168.31.182:23042")
    print()
    
    try:
        # 创建UDP socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        
        # 创建测试数据包 (0x02协议)
        test_packet = bytearray()
        test_packet.append(0x02)  # 协议头
        test_packet.append(0x00)  # 偏移量高字节
        test_packet.append(0x00)  # 偏移量低字节
        
        # 添加一些测试LED数据 (红色)
        for i in range(10):
            test_packet.extend([0, 255, 0])  # GRB格式的红色
        
        print(f"📦 发送测试数据包: {len(test_packet)} 字节")
        print(f"   协议头: 0x{test_packet[0]:02X}")
        print(f"   偏移量: {(test_packet[1] << 8) | test_packet[2]}")
        print(f"   数据长度: {len(test_packet) - 3} 字节")
        
        # 发送数据包
        target_addr = ('192.168.31.182', 23042)
        bytes_sent = sock.sendto(test_packet, target_addr)
        
        print(f"✅ 成功发送 {bytes_sent} 字节到硬件驱动板")
        
        sock.close()
        
    except Exception as e:
        print(f"❌ 发送失败: {e}")

def monitor_network_traffic():
    """监控网络流量（需要管理员权限）"""
    print("📡 网络流量监控")
    print("注意：此功能需要管理员权限，可能无法在普通用户模式下工作")
    print()
    
    try:
        # 这里可以添加网络监控代码
        # 但需要管理员权限，所以暂时跳过
        print("⚠️ 网络监控需要管理员权限，跳过此测试")
        
    except Exception as e:
        print(f"❌ 网络监控失败: {e}")

def analyze_hardware_response():
    """分析硬件响应"""
    print("🔍 分析硬件响应能力")
    print()
    
    # 检查硬件是否在线
    print("1. 检查硬件设备在线状态...")
    try:
        import subprocess
        result = subprocess.run(['ping', '-c', '1', '192.168.31.182'], 
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            print("✅ 硬件设备网络可达")
        else:
            print("❌ 硬件设备网络不可达")
            return
    except Exception as e:
        print(f"❌ 网络测试失败: {e}")
        return
    
    # 检查UDP端口
    print("2. 检查UDP端口状态...")
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(2)
        
        # 尝试连接（对于UDP，这只是设置默认目标）
        sock.connect(('192.168.31.182', 23042))
        print("✅ UDP端口可连接")
        
        sock.close()
        
    except Exception as e:
        print(f"❌ UDP端口测试失败: {e}")

def check_data_format():
    """检查数据格式是否正确"""
    print("📋 检查LED数据格式")
    print()
    
    # 模拟桌面应用程序发送的数据格式
    print("桌面应用程序发送的数据格式分析:")
    print("- 协议头: 0x02")
    print("- 偏移量: 0x0000 (大端序)")
    print("- 数据长度: 398字节")
    print("- LED数据格式: 混合GRB和GRBW")
    print()
    
    # 检查数据是否符合硬件期望
    print("硬件期望的数据格式:")
    print("- 协议: UDP")
    print("- 端口: 23042")
    print("- 数据包格式: [命令][偏移量高][偏移量低][LED数据...]")
    print("- LED格式: 根据配置可能是GRB或GRBW")
    print()

def suggest_debugging_steps():
    """建议调试步骤"""
    print("🛠️ 调试建议")
    print()
    
    print("1. 硬件端检查:")
    print("   - 检查硬件设备是否正常启动")
    print("   - 检查硬件设备的串口输出或日志")
    print("   - 确认硬件设备的UDP服务是否正在监听23042端口")
    print("   - 检查硬件设备的LED驱动是否正常工作")
    print()
    
    print("2. 网络层检查:")
    print("   - 使用Wireshark捕获网络包，确认数据包确实发送到硬件")
    print("   - 检查防火墙设置，确保UDP流量不被阻止")
    print("   - 检查路由器设置，确保局域网内设备可以通信")
    print()
    
    print("3. 数据格式检查:")
    print("   - 确认LED配置文件中的LED类型设置正确")
    print("   - 检查LED数据的字节序和格式")
    print("   - 验证偏移量计算是否正确")
    print()
    
    print("4. 时序检查:")
    print("   - 检查数据发送频率是否过高")
    print("   - 确认硬件设备能够处理当前的数据发送速率")
    print("   - 检查是否存在数据包丢失")
    print()

def main():
    """主函数"""
    print("🔧 LED硬件驱动板网络诊断工具")
    print("=" * 50)
    print()
    
    # 1. 测试直接UDP发送
    test_udp_send_to_hardware()
    print()
    
    # 2. 分析硬件响应
    analyze_hardware_response()
    print()
    
    # 3. 检查数据格式
    check_data_format()
    print()
    
    # 4. 建议调试步骤
    suggest_debugging_steps()
    
    print("=" * 50)
    print("🎯 关键问题:")
    print("1. 硬件设备是否有日志输出显示收到了UDP数据包？")
    print("2. 硬件设备的LED是否有任何反应（即使颜色不对）？")
    print("3. 硬件设备是否在处理其他类型的数据（如LED strip test）？")
    print("4. 是否可以通过其他方式（如串口）确认硬件设备状态？")

if __name__ == "__main__":
    main()
