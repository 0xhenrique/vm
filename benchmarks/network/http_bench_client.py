#!/usr/bin/env python3
"""
HTTP benchmark client supporting both regular and keep-alive connections.
Alternative to ab/wrk when those tools are not available.

Usage:
    python3 http_bench_client.py [url] [requests] [concurrency] [--keepalive]
"""

import sys
import time
import urllib.request
import urllib.error
import http.client
from urllib.parse import urlparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import statistics

def make_request(url):
    """Make a single HTTP request (new connection each time)"""
    start = time.perf_counter()
    try:
        with urllib.request.urlopen(url, timeout=5) as response:
            response.read()
        elapsed = (time.perf_counter() - start) * 1000
        return True, elapsed
    except Exception as e:
        elapsed = (time.perf_counter() - start) * 1000
        return False, elapsed

def make_keepalive_requests(host, port, path, num_requests):
    """Make multiple requests on a single keep-alive connection"""
    latencies = []
    successes = 0
    failures = 0

    try:
        conn = http.client.HTTPConnection(host, port, timeout=5)
        for _ in range(num_requests):
            start = time.perf_counter()
            try:
                conn.request("GET", path, headers={"Connection": "keep-alive"})
                response = conn.getresponse()
                response.read()
                elapsed = (time.perf_counter() - start) * 1000
                latencies.append(elapsed)
                successes += 1
            except Exception as e:
                elapsed = (time.perf_counter() - start) * 1000
                failures += 1
                # Try to reconnect
                try:
                    conn.close()
                    conn = http.client.HTTPConnection(host, port, timeout=5)
                except:
                    pass
        conn.close()
    except Exception as e:
        failures += num_requests - successes

    return successes, failures, latencies

def run_benchmark(url, total_requests, concurrency, keepalive=False):
    """Run HTTP benchmark with given parameters"""
    parsed = urlparse(url)
    host = parsed.hostname or "localhost"
    port = parsed.port or 80
    path = parsed.path or "/"

    mode = "Keep-Alive" if keepalive else "New Connection"
    print(f"HTTP Benchmark Client (Python) - {mode}")
    print("=" * 50)
    print(f"URL: {url}")
    print(f"Total Requests: {total_requests}")
    print(f"Concurrency: {concurrency}")
    print()

    successes = 0
    failures = 0
    latencies = []

    start_time = time.perf_counter()

    if keepalive:
        # Keep-alive mode: each worker makes multiple requests on one connection
        requests_per_worker = total_requests // concurrency
        remainder = total_requests % concurrency

        with ThreadPoolExecutor(max_workers=concurrency) as executor:
            futures = []
            for i in range(concurrency):
                # Distribute extra requests to first workers
                worker_requests = requests_per_worker + (1 if i < remainder else 0)
                if worker_requests > 0:
                    futures.append(executor.submit(
                        make_keepalive_requests, host, port, path, worker_requests
                    ))

            for future in as_completed(futures):
                s, f, lats = future.result()
                successes += s
                failures += f
                latencies.extend(lats)
    else:
        # Non-keep-alive mode: new connection per request
        with ThreadPoolExecutor(max_workers=concurrency) as executor:
            futures = [executor.submit(make_request, url) for _ in range(total_requests)]

            for future in as_completed(futures):
                success, latency = future.result()
                if success:
                    successes += 1
                    latencies.append(latency)
                else:
                    failures += 1

    total_time = time.perf_counter() - start_time

    print("Results:")
    print("-" * 50)
    print(f"Successful requests: {successes}")
    print(f"Failed requests: {failures}")
    print(f"Total time: {total_time:.3f} s")

    if successes > 0:
        rps = successes / total_time
        avg_latency = statistics.mean(latencies)
        median_latency = statistics.median(latencies)

        print(f"Requests/sec: {rps:.2f}")
        print(f"Latency (mean): {avg_latency:.2f} ms")
        print(f"Latency (median): {median_latency:.2f} ms")

        if len(latencies) > 1:
            p95 = sorted(latencies)[int(len(latencies) * 0.95)]
            p99 = sorted(latencies)[int(len(latencies) * 0.99)]
            print(f"Latency (p95): {p95:.2f} ms")
            print(f"Latency (p99): {p99:.2f} ms")

        return rps
    return 0

def main():
    # Parse arguments
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    keepalive = '--keepalive' in sys.argv or '-k' in sys.argv

    url = args[0] if len(args) > 0 else "http://localhost:8080/"
    total_requests = int(args[1]) if len(args) > 1 else 1000
    concurrency = int(args[2]) if len(args) > 2 else 10

    run_benchmark(url, total_requests, concurrency, keepalive)

if __name__ == "__main__":
    main()
