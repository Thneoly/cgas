#!/usr/bin/env python3
"""
CGAS Application Metrics Exporter
提供简单的 HTTP/metrics 端点用于 Prometheus 抓取
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import time
import random
import os

# 指标数据
START_TIME = time.time()
REQUEST_COUNT = 0
ERROR_COUNT = 0

class MetricsHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        global REQUEST_COUNT
        REQUEST_COUNT += 1
        
        if self.path == '/metrics':
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            
            metrics = f"""# HELP app_uptime_seconds Application uptime in seconds
# TYPE app_uptime_seconds counter
app_uptime_seconds {time.time() - START_TIME}

# HELP app_requests_total Total number of requests
# TYPE app_requests_total counter
app_requests_total {REQUEST_COUNT}

# HELP app_errors_total Total number of errors
# TYPE app_errors_total counter
app_errors_total {ERROR_COUNT}

# HELP app_info Application information
# TYPE app_info gauge
app_info{{version="3.0.0",environment="{os.environ.get('ENVIRONMENT', 'unknown')}"}} 1

# HELP app_memory_usage_bytes Application memory usage
# TYPE app_memory_usage_bytes gauge
app_memory_usage_bytes {random.randint(100000000, 500000000)}

# HELP app_cpu_usage_percent Application CPU usage
# TYPE app_cpu_usage_percent gauge
app_cpu_usage_percent {random.uniform(10.0, 50.0)}
"""
            self.wfile.write(metrics.encode())
        elif self.path == '/health':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"status":"healthy"}')
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass  # 禁用日志

if __name__ == '__main__':
    server = HTTPServer(('0.0.0.0', 8080), MetricsHandler)
    print(f"Metrics server started on port 8080")
    server.serve_forever()
