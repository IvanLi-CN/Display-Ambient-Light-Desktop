#!/usr/bin/env python3
"""
数据包拦截器 - 拦截并分析发送到硬件的数据包
"""

import socket
import threading
import time
from datetime import datetime

class PacketInterceptor:
    def __init__(self, target_ip="192.168.31.182", target_port=23042, listen_port=8889):
        self.target_ip = target_ip
        self.target_port = target_port
        self.listen_port = listen_port
        self.running = False
        self.packet_count = 0
        
    def start_interceptor(self):
        """启动数据包拦截器"""
        print(f"🔧 启动数据包拦截器")
        print(f"监听端口: {self.listen_port}")
        print(f"转发目标: {self.target_ip}:{self.target_port}")
        print("=" * 60)
        
        try:
            # 创建监听socket
            listen_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            listen_sock.bind(('0.0.0.0', self.listen_port))
            
            # 创建转发socket
            forward_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            
            self.running = True
            print(f"✅ 拦截器启动成功，等待数据包...")
            print(f"💡 提示：修改桌面应用程序目标地址为 127.0.0.1:{self.listen_port} 来测试")
            print()
            
            while self.running:
                try:
                    # 接收数据包
                    data, addr = listen_sock.recvfrom(65536)
                    self.packet_count += 1
                    
                    # 分析数据包
                    self.analyze_packet(data, addr)
                    
                    # 转发到真实硬件
                    forward_sock.sendto(data, (self.target_ip, self.target_port))
                    print(f"📤 已转发到硬件设备")
                    print("-" * 40)
                    
                except socket.timeout:
                    continue
                except Exception as e:
                    print(f"❌ 处理数据包时出错: {e}")
                    
        except Exception as e:
            print(f"❌ 启动拦截器失败: {e}")
        finally:
            try:
                listen_sock.close()
                forward_sock.close()
            except:
                pass
    
    def analyze_packet(self, data, addr):
        """分析数据包内容"""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        
        print(f"📦 数据包 #{self.packet_count} - {timestamp}")
        print(f"📍 来源地址: {addr[0]}:{addr[1]}")
        print(f"📏 数据长度: {len(data)} 字节")
        
        if len(data) >= 3:
            protocol = data[0]
            offset = (data[1] << 8) | data[2]
            led_data_len = len(data) - 3
            
            print(f"🔍 协议命令: 0x{protocol:02X}")
            print(f"📍 字节偏移量: {offset}")
            print(f"🎨 LED数据长度: {led_data_len} 字节")
            
            # 分析LED数据
            if led_data_len > 0:
                self.analyze_led_data(data[3:])
            
            # 显示原始数据的前32字节
            hex_data = ' '.join(f'{b:02X}' for b in data[:min(32, len(data))])
            ascii_data = ''.join(chr(b) if 32 <= b <= 126 else '.' for b in data[:min(32, len(data))])
            print(f"📋 原始数据 (前{min(32, len(data))}字节):")
            print(f"   {hex_data}")
            print(f"   {ascii_data}")
        else:
            print("⚠️ 数据包太短，无法解析")
    
    def analyze_led_data(self, led_data):
        """分析LED数据"""
        data_len = len(led_data)
        
        # 尝试按GRB格式解析
        if data_len % 3 == 0:
            led_count_grb = data_len // 3
            print(f"   📊 按GRB格式: {led_count_grb} 个LED")
            
            # 显示前5个LED的颜色
            for i in range(min(5, led_count_grb)):
                offset = i * 3
                g, r, b = led_data[offset], led_data[offset+1], led_data[offset+2]
                print(f"      LED {i+1}: G={g:3d}, R={r:3d}, B={b:3d} (#{r:02X}{g:02X}{b:02X})")
        
        # 尝试按GRBW格式解析
        if data_len % 4 == 0:
            led_count_grbw = data_len // 4
            print(f"   📊 按GRBW格式: {led_count_grbw} 个LED")
            
            # 显示前5个LED的颜色
            for i in range(min(5, led_count_grbw)):
                offset = i * 4
                g, r, b, w = led_data[offset], led_data[offset+1], led_data[offset+2], led_data[offset+3]
                print(f"      LED {i+1}: G={g:3d}, R={r:3d}, B={b:3d}, W={w:3d}")
    
    def stop(self):
        """停止拦截器"""
        self.running = False
        print("\n🛑 正在停止拦截器...")

def main():
    """主函数"""
    print("🔧 LED数据包拦截分析工具")
    print("此工具可以拦截、分析并转发LED数据包")
    print()
    
    interceptor = PacketInterceptor()
    
    try:
        # 启动拦截器
        interceptor.start_interceptor()
        
    except KeyboardInterrupt:
        print("\n⌨️ 收到中断信号")
    finally:
        interceptor.stop()
        print("✅ 拦截器已停止")

if __name__ == "__main__":
    main()
