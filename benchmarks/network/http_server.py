#!/usr/bin/env python3
"""
Simple HTTP server in Python for benchmarking
Equivalent to the Lisp http_server_simple.lisp
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import sys

class SimpleHandler(BaseHTTPRequestHandler):
    request_count = 0
    max_requests = 100000

    def do_GET(self):
        SimpleHandler.request_count += 1

        self.send_response(200)
        self.send_header('Content-Type', 'text/plain')
        body = b'Hello from Python!'
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

        if SimpleHandler.request_count >= SimpleHandler.max_requests:
            sys.exit(0)

    def log_message(self, format, *args):
        pass  # Suppress logging for fair comparison

if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    server = HTTPServer(('0.0.0.0', port), SimpleHandler)
    print(f"Python HTTP server listening on port {port}")
    server.serve_forever()
