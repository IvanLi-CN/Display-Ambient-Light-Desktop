#!/usr/bin/env python3
"""
直接测试LED数据发送 - 绕过前端界面
通过UDP直接发送LED数据包到虚拟驱动板，模拟桌面应用程序的行为
"""

import socket
import struct
import time

def create_led_test_data():
    """创建LED测试数据"""
    # 模拟4个LED灯带的配置
    strips = [
        {'border': 'bottom', 'count': 38, 'ledType': 'SK6812', 'sequence': 1},
        {'border': 'right', 'count': 22, 'ledType': 'WS2812B', 'sequence': 2},
        {'border': 'top', 'count': 38, 'ledType': 'SK6812', 'sequence': 3},
        {'border': 'left', 'count': 22, 'ledType': 'WS2812B', 'sequence': 4}
    ]

    # 生成边框测试颜色 - 优化相邻颜色差异的8色方案
    border_colors = {
        'bottom': [{'r': 255, 'g': 100, 'b': 0}, {'r': 255, 'g': 255, 'b': 0}],   # 深橙色 + 黄色
        'right': [{'r': 0, 'g': 255, 'b': 0}, {'r': 0, 'g': 255, 'b': 255}],      # 纯绿色 + 青色
        'top': [{'r': 0, 'g': 100, 'b': 255}, {'r': 150, 'g': 0, 'b': 255}],      # 蓝色 + 紫色
        'left': [{'r': 255, 'g': 0, 'b': 150}, {'r': 255, 'g': 0, 'b': 0}]        # 玫红色 + 红色
    }

    all_color_bytes = []
    
    # 按序列号排序
    strips.sort(key=lambda x: x['sequence'])
    
    for strip in strips:
        colors = border_colors[strip['border']]
        half_count = strip['count'] // 2
        
        print(f"生成 {strip['border']} 边框数据: {strip['count']} 个LED ({strip['ledType']})")
        
        # 前半部分使用第一种颜色
        for i in range(half_count):
            color = colors[0]
            if strip['ledType'] == 'SK6812':
                all_color_bytes.extend([color['g'], color['r'], color['b'], 255])  # GRBW
            else:
                all_color_bytes.extend([color['g'], color['r'], color['b']])  # GRB
        
        # 后半部分使用第二种颜色
        for i in range(half_count, strip['count']):
            color = colors[1]
            if strip['ledType'] == 'SK6812':
                all_color_bytes.extend([color['g'], color['r'], color['b'], 255])  # GRBW
            else:
                all_color_bytes.extend([color['g'], color['r'], color['b']])  # GRB
    
    print(f"总共生成 {len(all_color_bytes)} 字节的LED数据")
    return all_color_bytes

def create_led_packet(offset, data):
    """创建LED数据包 (0x02协议)"""
    packet = bytearray()
    packet.append(0x02)  # 协议头
    packet.append((offset >> 8) & 0xFF)  # 偏移量高字节
    packet.append(offset & 0xFF)  # 偏移量低字节
    packet.extend(data)  # LED数据
    return packet

def send_led_data_to_virtual_board(data, target_host='127.0.0.1', target_port=8888):
    """发送LED数据到虚拟驱动板"""
    try:
        # 创建UDP socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        
        # 创建LED数据包
        packet = create_led_packet(0, data)
        
        print(f"📤 发送 {len(packet)} 字节数据包到 {target_host}:{target_port}")
        print(f"   协议头: 0x{packet[0]:02X}")
        print(f"   偏移量: {(packet[1] << 8) | packet[2]}")
        print(f"   数据长度: {len(packet) - 3} 字节")
        
        # 发送数据包
        sock.sendto(packet, (target_host, target_port))
        
        print(f"✅ 数据包发送成功")
        
        sock.close()
        
    except Exception as e:
        print(f"❌ 发送数据包失败: {e}")

def send_multiple_packets(data, max_packet_size=396, target_host='127.0.0.1', target_port=8888):
    """发送多个数据包（模拟大数据的分包发送）"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        
        offset = 0
        packet_count = 0
        
        while offset < len(data):
            # 计算当前包的数据大小
            chunk_size = min(max_packet_size, len(data) - offset)
            chunk = data[offset:offset + chunk_size]
            
            # 创建数据包
            packet = create_led_packet(offset, chunk)
            
            packet_count += 1
            print(f"📤 发送数据包 {packet_count}: 偏移量={offset}, 大小={len(chunk)} 字节")
            
            # 发送数据包
            sock.sendto(packet, (target_host, target_port))
            
            offset += chunk_size
            
            # 稍微延迟以避免网络拥塞
            time.sleep(0.01)
        
        print(f"✅ 所有数据包发送完成: {packet_count} 个包, 总计 {len(data)} 字节")
        
        sock.close()
        
    except Exception as e:
        print(f"❌ 发送数据包失败: {e}")

def test_real_hardware(data, target_host='192.168.31.182', target_port=23042):
    """测试发送数据到真实硬件"""
    print(f"\n🔧 测试发送到真实硬件: {target_host}:{target_port}")
    send_led_data_to_virtual_board(data, target_host, target_port)

def main():
    """主函数"""
    print("🔧 LED数据直接测试工具")
    print("直接通过UDP发送LED数据包，绕过前端界面")
    print()
    
    # 1. 生成测试数据
    print("📦 生成LED测试数据...")
    test_data = create_led_test_data()
    
    print(f"\n📊 数据统计:")
    print(f"   总字节数: {len(test_data)}")
    print(f"   前10字节: {test_data[:10]}")
    print(f"   后10字节: {test_data[-10:]}")
    
    # 2. 发送到虚拟驱动板
    print(f"\n🎯 发送到虚拟驱动板...")
    send_led_data_to_virtual_board(test_data)
    
    # 3. 测试分包发送
    print(f"\n📦 测试分包发送...")
    send_multiple_packets(test_data, max_packet_size=100)
    
    # 4. 发送到真实硬件（如果需要）
    try_real_hardware = input(f"\n❓ 是否也发送到真实硬件? (y/N): ").strip().lower()
    if try_real_hardware == 'y':
        test_real_hardware(test_data)
    
    print(f"\n🎉 测试完成")

if __name__ == "__main__":
    main()
