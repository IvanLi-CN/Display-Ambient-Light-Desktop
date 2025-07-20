#!/usr/bin/env python3
"""
虚拟LED驱动板 - 用于调试LED数据传输问题
接收来自桌面应用程序的LED数据并分析协议内容
支持mDNS服务发布，模拟真实硬件设备
"""

import socket
import threading
import time
from datetime import datetime
import struct
import sys

# 尝试导入zeroconf库用于mDNS服务发布
try:
    from zeroconf import ServiceInfo, Zeroconf
    MDNS_AVAILABLE = True
except ImportError:
    MDNS_AVAILABLE = False
    print("⚠️ 警告: zeroconf库未安装，mDNS服务发布功能不可用")
    print("💡 安装方法: pip install zeroconf")
    print("🔧 虚拟驱动板将以UDP-only模式运行")

class VirtualLedBoard:
    def __init__(self, host='0.0.0.0', port=8888, device_name="Virtual LED Board"):
        self.host = host
        self.port = port
        self.device_name = device_name
        self.socket = None
        self.running = False
        self.packet_count = 0
        self.last_packet_time = None

        # mDNS相关
        self.zeroconf = None
        self.service_info = None
        self.mdns_enabled = MDNS_AVAILABLE
        
    def start(self):
        """启动虚拟驱动板服务器"""
        try:
            # 启动UDP服务器
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self.socket.bind((self.host, self.port))
            self.running = True

            print(f"🚀 虚拟LED驱动板启动成功")
            print(f"📡 监听地址: {self.host}:{self.port}")
            print(f"🏷️  设备名称: {self.device_name}")
            print(f"⏰ 启动时间: {datetime.now().strftime('%H:%M:%S')}")

            # 启动mDNS服务发布
            if self.mdns_enabled:
                if self._start_mdns_service():
                    print(f"📢 mDNS服务发布成功: _ambient_light._udp.local.")
                else:
                    print(f"❌ mDNS服务发布失败，继续以UDP-only模式运行")
            else:
                print(f"⚠️ mDNS功能未启用，仅UDP模式运行")

            print("=" * 60)

            # 启动接收线程
            receive_thread = threading.Thread(target=self._receive_loop)
            receive_thread.daemon = True
            receive_thread.start()

            return True

        except Exception as e:
            print(f"❌ 启动虚拟驱动板失败: {e}")
            return False
    
    def stop(self):
        """停止虚拟驱动板"""
        self.running = False

        # 停止mDNS服务
        if self.zeroconf and self.service_info:
            try:
                self.zeroconf.unregister_service(self.service_info)
                self.zeroconf.close()
                print("📢 mDNS服务已注销")
            except Exception as e:
                print(f"⚠️ mDNS服务注销失败: {e}")

        # 停止UDP服务器
        if self.socket:
            self.socket.close()
        print("🛑 虚拟LED驱动板已停止")

    def _start_mdns_service(self):
        """启动mDNS服务发布"""
        try:
            # 获取本机IP地址
            local_ip = self._get_local_ip()
            if not local_ip:
                print("❌ 无法获取本机IP地址")
                return False

            # 创建服务信息 - 使用兼容的服务类型
            # zeroconf库不支持服务类型中的下划线，所以使用连字符
            service_type = "_ambient-light._udp.local."
            service_name = "Virtual-LED-Board-Debug"

            # TXT记录，模拟真实硬件设备的属性
            txt_properties = {
                'protocol': 'udp',
                'max_leds': '500',
                'type': 'ambient_light',
                'device': 'virtual_board',
                'version': '1.0',
                'debug': 'true'
            }

            # 使用简化的ServiceInfo创建方式
            self.service_info = ServiceInfo(
                type_=service_type,
                name=f"{service_name}.{service_type}",
                addresses=[socket.inet_aton(local_ip)],
                port=self.port,
                properties=txt_properties
            )

            # 启动Zeroconf服务
            self.zeroconf = Zeroconf()
            self.zeroconf.register_service(self.service_info)

            print(f"📍 本机IP地址: {local_ip}")
            print(f"🏷️  服务类型: {service_type}")
            print(f"🏷️  服务名称: {service_name}")
            print(f"📋 TXT属性: {txt_properties}")

            return True

        except Exception as e:
            print(f"❌ mDNS服务启动失败: {e}")
            return False

    def _get_local_ip(self):
        """获取本机IP地址"""
        try:
            # 创建一个UDP socket连接到外部地址来获取本机IP
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
                s.connect(("8.8.8.8", 80))
                return s.getsockname()[0]
        except Exception:
            try:
                # 备用方法：通过hostname获取
                import socket
                hostname = socket.gethostname()
                return socket.gethostbyname(hostname)
            except Exception:
                return None

    def _receive_loop(self):
        """接收数据的主循环"""
        while self.running:
            try:
                data, addr = self.socket.recvfrom(1024)
                self._process_packet(data, addr)
                
            except socket.timeout:
                continue
            except Exception as e:
                if self.running:
                    print(f"❌ 接收数据错误: {e}")
                break
    
    def _process_packet(self, data, addr):
        """处理接收到的数据包"""
        self.packet_count += 1
        current_time = datetime.now()
        
        # 计算时间间隔
        time_interval = ""
        if self.last_packet_time:
            interval_ms = (current_time - self.last_packet_time).total_seconds() * 1000
            time_interval = f" (+{interval_ms:.1f}ms)"
        self.last_packet_time = current_time
        
        print(f"\n📦 数据包 #{self.packet_count} - {current_time.strftime('%H:%M:%S.%f')[:-3]}{time_interval}")
        print(f"📍 来源地址: {addr[0]}:{addr[1]}")
        print(f"📏 数据长度: {len(data)} 字节")
        
        if len(data) == 0:
            print("⚠️ 空数据包")
            return
        
        # 分析协议头
        command = data[0]
        print(f"🔍 协议命令: 0x{command:02X}")
        
        if command == 0x01:
            print("💓 心跳检查命令")
            self._send_heartbeat_response(addr)
        elif command == 0x02:
            print("🌈 LED数据命令")
            self._analyze_led_data_packet(data)
            self._send_ack_response(addr)
        elif command == 0x03:
            print("⚙️ 配置命令")
            self._send_ack_response(addr)
        elif command == 0x04:
            print("🔗 连接检查命令")
            self._send_connection_response(addr)
        else:
            print(f"❓ 未知命令: 0x{command:02X}")
            self._print_hex_dump(data)
    
    def _analyze_led_data_packet(self, data):
        """分析LED数据包 (0x02协议)"""
        if len(data) < 3:
            print("❌ LED数据包长度不足")
            return
        
        # 解析偏移量 (大端序)
        offset = (data[1] << 8) | data[2]
        led_data = data[3:]
        
        print(f"📍 字节偏移量: {offset}")
        print(f"🎨 LED数据长度: {len(led_data)} 字节")
        
        # 分析LED数据
        if len(led_data) > 0:
            self._analyze_led_colors(led_data, offset)
        
        # 显示原始数据的十六进制转储
        print(f"📋 原始数据 (前32字节):")
        self._print_hex_dump(data[:min(32, len(data))])

    def _send_heartbeat_response(self, addr):
        """发送心跳响应"""
        try:
            # 简单的心跳响应，返回0x01表示设备活跃
            response = bytes([0x01])
            self.socket.sendto(response, addr)
            print("💓 已发送心跳响应")
        except Exception as e:
            print(f"❌ 发送心跳响应失败: {e}")

    def _send_ack_response(self, addr):
        """发送确认响应"""
        try:
            # 发送0x00表示成功接收
            response = bytes([0x00])
            self.socket.sendto(response, addr)
            print("✅ 已发送确认响应")
        except Exception as e:
            print(f"❌ 发送确认响应失败: {e}")

    def _send_connection_response(self, addr):
        """发送连接检查响应"""
        try:
            # 发送设备信息响应
            response = bytes([0x04, 0x01])  # 0x04命令 + 0x01表示连接成功
            self.socket.sendto(response, addr)
            print("🔗 已发送连接响应")
        except Exception as e:
            print(f"❌ 发送连接响应失败: {e}")

    def _analyze_led_colors(self, led_data, offset):
        """分析LED颜色数据"""
        # 尝试按不同格式解析
        print(f"\n🎨 LED颜色分析:")
        
        # 按3字节(GRB)格式分析
        if len(led_data) % 3 == 0:
            led_count_grb = len(led_data) // 3
            print(f"   📊 按GRB格式: {led_count_grb} 个LED")
            for i in range(min(5, led_count_grb)):  # 只显示前5个LED
                start_idx = i * 3
                g, r, b = led_data[start_idx:start_idx+3]
                print(f"      LED {i+1}: G={g:3d}, R={r:3d}, B={b:3d} (#{r:02X}{g:02X}{b:02X})")
        
        # 按4字节(GRBW)格式分析
        if len(led_data) % 4 == 0:
            led_count_grbw = len(led_data) // 4
            print(f"   📊 按GRBW格式: {led_count_grbw} 个LED")
            for i in range(min(5, led_count_grbw)):  # 只显示前5个LED
                start_idx = i * 4
                g, r, b, w = led_data[start_idx:start_idx+4]
                print(f"      LED {i+1}: G={g:3d}, R={r:3d}, B={b:3d}, W={w:3d}")
    
    def _print_hex_dump(self, data):
        """打印十六进制转储"""
        for i in range(0, len(data), 16):
            hex_part = ' '.join(f'{b:02X}' for b in data[i:i+16])
            ascii_part = ''.join(chr(b) if 32 <= b <= 126 else '.' for b in data[i:i+16])
            print(f"   {i:04X}: {hex_part:<48} |{ascii_part}|")

def main():
    """主函数"""
    print("🔧 虚拟LED驱动板调试工具")
    print("用于接收和分析来自桌面应用程序的LED数据")
    print("支持mDNS服务发布，模拟真实硬件设备")
    print()

    # 检查zeroconf依赖
    if not MDNS_AVAILABLE:
        print("📦 依赖检查:")
        print("   ❌ zeroconf库未安装")
        print("   💡 安装命令: pip install zeroconf")
        print("   🔧 将以UDP-only模式运行")
        print()

    # 创建虚拟驱动板实例
    board = VirtualLedBoard(device_name="Virtual LED Board Debug")

    if not board.start():
        return

    try:
        print("🎯 等待LED数据包...")
        print("💡 提示: 在桌面应用程序中访问LED配置界面来发送测试数据")
        if MDNS_AVAILABLE:
            print("📢 mDNS服务已发布，桌面应用程序应该能自动发现此虚拟设备")
        else:
            print("⚠️ mDNS服务未启用，需要手动配置桌面应用程序连接到虚拟设备")
        print("⌨️  按 Ctrl+C 停止")
        print()

        # 保持运行
        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        print("\n\n⌨️ 收到停止信号")
    finally:
        board.stop()

if __name__ == "__main__":
    main()
