;; HTTP Server with Multi-threaded Handler Pool
;; Uses parallel request handling for improved throughput under high load
;; Handles 100000 requests then exits (enough for any benchmark run)

;; Handler function: receives request hashmap, returns response hashmap
(defun handle-request
  ((request)
    (hash-map "status" 200 "body" "Hello from Lisp!" "keep-alive" true)))

;; Main server using parallel handler pool
(defun run-parallel-server
  ((port num-workers max-requests)
    (let ((listener (http-listen-shared port)))
      ;; http-serve-parallel takes: listener, handler, num_workers, max_requests
      ;; Returns total requests handled
      (http-serve-parallel listener handle-request num-workers max-requests))))

;; Run with 4 workers by default (can be configured based on CPU cores)
(run-parallel-server 8080 4 100000)
