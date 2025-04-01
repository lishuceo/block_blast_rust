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
    
    def do_GET(self):
        # 处理API请求
        if self.path == '/api/games':
            self.send_json_response('games.json')
        # 添加对根路径的处理
        elif self.path == '/' or self.path == '/index.html':
            self.send_game_list_page()
        else:
            # 处理正常的静态文件请求
            super().do_GET()
    
    def send_json_response(self, json_file):
        try:
            # 构建JSON文件的完整路径
            file_path = os.path.join(self.directory, json_file)
            
            # 检查文件是否存在
            if not os.path.exists(file_path):
                self.send_response(404)
                self.send_header('Content-type', 'text/plain')
                self.end_headers()
                self.wfile.write(f"File not found: {json_file}".encode('utf-8'))
                return
            
            # 使用utf-8-sig读取JSON文件，自动处理BOM标记
            with open(file_path, 'r', encoding='utf-8-sig') as f:
                data = json.load(f)
            
            # 准备响应
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')  # 允许跨域请求
            self.end_headers()
            
            # 发送JSON数据
            self.wfile.write(json.dumps(data, ensure_ascii=False).encode('utf-8'))
        except Exception as e:
            self.send_response(500)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            self.wfile.write(f"Error processing request: {str(e)}".encode('utf-8'))

    def send_game_list_page(self):
        try:
            # 读取游戏列表数据
            file_path = os.path.join(self.directory, 'games.json')
            
            # 检查文件是否存在
            if not os.path.exists(file_path):
                self.send_error(404, "Games list not found")
                return
            
            # 读取游戏数据
            with open(file_path, 'r', encoding='utf-8-sig') as f:
                game_data = json.load(f)
            
            # 生成HTML页面
            html = """
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>迷你游戏列表</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f0f0f0;
        }
        h1 {
            color: #333;
            text-align: center;
            padding-bottom: 20px;
            border-bottom: 1px solid #ddd;
        }
        .game-list {
            list-style: none;
            padding: 0;
        }
        .game-item {
            background-color: white;
            border-radius: 8px;
            margin-bottom: 15px;
            padding: 15px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            transition: transform 0.2s ease;
        }
        .game-item:hover {
            transform: translateY(-3px);
            box-shadow: 0 4px 8px rgba(0,0,0,0.15);
        }
        .game-link {
            display: block;
            color: #2a80eb;
            font-size: 18px;
            text-decoration: none;
            padding: 10px 0;
        }
        .game-link:hover {
            color: #1c60b3;
        }
        .footer {
            text-align: center;
            margin-top: 30px;
            color: #666;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <h1>迷你游戏列表</h1>
    <ul class="game-list">
"""
            
            # 添加游戏列表项
            if 'games' in game_data and len(game_data['games']) > 0:
                for game in game_data['games']:
                    html += f"""
        <li class="game-item">
            <a class="game-link" href="{game['url']}" target="_blank">{game['name']}</a>
        </li>
"""
            else:
                html += """
        <li class="game-item">暂无游戏</li>
"""
                
            # 完成HTML页面
            html += """
    </ul>
    <div class="footer">
        <p>WebAssembly 游戏演示</p>
    </div>
</body>
</html>
"""
            
            # 发送HTML响应
            self.send_response(200)
            self.send_header('Content-type', 'text/html; charset=utf-8')
            self.end_headers()
            self.wfile.write(html.encode('utf-8'))
            
        except Exception as e:
            self.send_response(500)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            self.wfile.write(f"Error generating game list page: {str(e)}".encode('utf-8'))

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

