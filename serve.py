import http.server, socketserver
import functools

PORT = 8000
Handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory='web')

print(f"启动服务器在端口 {PORT}...")
print(f"请访问 http://localhost:{PORT}/")
print("按Ctrl+C退出服务器")

with socketserver.TCPServer(("", PORT), Handler) as httpd:
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n服务器已停止")
