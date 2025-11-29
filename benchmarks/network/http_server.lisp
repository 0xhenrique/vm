;; HTTP Server for Benchmarking with Keep-Alive Support
;; Equivalent to Python/Lua/Ruby versions
;; Handles 100000 requests then exits (enough for any benchmark run)

;; Handle a keep-alive connection (inner loop)
;; Returns count after processing all requests on this connection
(defun handle-keep-alive-connection
  ((stream count max-requests)
    (loop ((c count))
      (if (>= c max-requests)
        c
        (let ((request (http-read-request stream)))
          (if (hashmap? request)
            ;; Valid request - send keep-alive response and continue
            (let ((response (hash-map "status" 200 "body" "Hello from Lisp!" "keep-alive" true)))
              (let ((y (http-send-response stream response)))
                (recur (+ c 1))))
            ;; Connection closed by client
            c))))))

;; Main server loop - accepts connections (outer loop)
(defun run-benchmark-server
  ((port max-requests)
    (let ((listener (http-listen port)))
      (loop ((count 0))
        (if (>= count max-requests)
          0
          (let ((stream (http-accept listener)))
            (let ((new-count (handle-keep-alive-connection stream count max-requests)))
              (let ((z (http-close stream)))
                (recur new-count)))))))))

(run-benchmark-server 8080 100000)
