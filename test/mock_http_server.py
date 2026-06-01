#!/usr/bin/env python3
import argparse
import http.server
import os
import socketserver
import sys


class ReusableTCPServer(socketserver.TCPServer):
    allow_reuse_address = True


def make_handler(path, mode, fail_count):
    class Handler(http.server.BaseHTTPRequestHandler):
        attempts = 0

        def log_message(self, fmt, *args):
            return

        def do_GET(self):
            type(self).attempts += 1
            attempt = type(self).attempts

            if self.path != "/data":
                self.send_error(404)
                return

            if mode == "404":
                self.send_error(404)
                return

            if mode == "503_then_ok" and attempt <= fail_count:
                self.send_response(503)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return

            if mode == "429_then_ok" and attempt <= fail_count:
                self.send_response(429)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return

            with open(path, "rb") as fh:
                data = fh.read()
            start = 0
            range_header = self.headers.get("Range")
            if range_header and range_header.startswith("bytes="):
                first = range_header[6:].split("-", 1)[0]
                try:
                    start = int(first)
                except ValueError:
                    start = 0
                if start < 0:
                    start = 0
                if start > len(data):
                    start = len(data)

            if mode == "drop_mid_transfer" and attempt == 1 and fail_count > 0 and start == 0:
                partial = data[start : start + max(1, len(data) // 3)]
                self.send_response(200)
                self.send_header("Content-Length", str(len(data) - start))
                self.end_headers()
                self.wfile.write(partial)
                self.wfile.flush()
                self.connection.shutdown(1)
                self.connection.close()
                return

            if start > 0:
                self.send_response(206)
                self.send_header("Content-Range", f"bytes {start}-{len(data) - 1}/{len(data)}")
            else:
                self.send_response(200)
            self.send_header("Content-Length", str(len(data) - start))
            self.end_headers()
            self.wfile.write(data[start:])

    return Handler


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True)
    parser.add_argument("--file", required=True)
    parser.add_argument("--fail-count", type=int, default=0)
    parser.add_argument("--port", type=int, default=0)
    args = parser.parse_args()

    handler = make_handler(args.file, args.mode, args.fail_count)
    with ReusableTCPServer(("127.0.0.1", args.port), handler) as httpd:
        print(httpd.server_address[1], flush=True)
        httpd.serve_forever()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(0)
    except BrokenPipeError:
        sys.exit(0)
