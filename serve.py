import http.server, socketserver
import functools
import json
import os
import ssl
import socket
import sys
import argparse
from pathlib import Path

# 解析命令行参数
parser = argparse.ArgumentParser(description='启动游戏Web服务器，支持HTTP或HTTPS')
parser.add_argument('--https', action='store_true', help='使用HTTPS模式启动服务器')
parser.add_argument('--port', type=int, default=8000, help='服务器端口号(默认HTTP:8000, HTTPS:8443)')
parser.add_argument('--cert', type=str, default="cert.crt", help='SSL证书文件路径')
parser.add_argument('--key', type=str, default="private.key", help='SSL私钥文件路径')
args = parser.parse_args()

# 根据参数设置端口
PORT = args.port if args.port else (8443 if args.https else 8000)

# 证书文件路径
CERT_FILE = args.cert
KEY_FILE = args.key

# 创建自定义请求处理器
class GameAPIHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        kwargs["directory"] = 'web'
        super().__init__(*args, **kwargs)
    
    # 重写end_headers方法，添加禁用缓存的HTTP头
    def end_headers(self):
        # 添加禁用缓存的HTTP头
        self.send_header('Cache-Control', 'no-store, no-cache, must-revalidate, max-age=0')
        self.send_header('Pragma', 'no-cache')
        self.send_header('Expires', '0')
        # 调用原始的end_headers方法
        super().end_headers()
    
# 获取本地IP地址
def get_local_ip():
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))  # 连接外部服务器
        ip = s.getsockname()[0]
        s.close()
        return ip
    except:
        return socket.gethostbyname(socket.gethostname())

local_ip = get_local_ip()

# 打印服务器信息
protocol = "HTTPS" if args.https else "HTTP"
print(f"启动{protocol}服务器在端口 {PORT}...")
print(f"请访问: {protocol.lower()}://localhost:{PORT}/")
print(f"局域网访问: {protocol.lower()}://{local_ip}:{PORT}/")
print(f"API数据: {protocol.lower()}://{local_ip}:{PORT}/api/games")
print("按Ctrl+C退出服务器")

# 创建HTTP(S)服务器
server = socketserver.ThreadingTCPServer(("0.0.0.0", PORT), GameAPIHandler)

# 如果是HTTPS模式，配置SSL
if args.https:
    try:
        # 检查证书文件是否存在
        if not os.path.exists(CERT_FILE) or not os.path.exists(KEY_FILE):
            print(f"错误: 证书文件不存在，请检查路径: {CERT_FILE}, {KEY_FILE}")
            sys.exit(1)
            
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(certfile=CERT_FILE, keyfile=KEY_FILE)
        server.socket = context.wrap_socket(server.socket, server_side=True)
        print(f"SSL配置已启用，使用证书: {CERT_FILE}")
    except Exception as e:
        print(f"SSL配置失败: {e}")
        sys.exit(1)

print("服务器启用多线程模式，支持并发连接")
try:
    server.serve_forever()
except KeyboardInterrupt:
    print("\n服务器已停止")
except Exception as e:
    print(f"服务器错误: {e}")

