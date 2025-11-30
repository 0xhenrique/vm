;; Minimal HTTP Server Example
;; Demonstrates Phase 14 HTTP functionality
;;
;; Usage:
;;   ./target/release/bytecomp examples/http_server_simple.lisp -o /tmp/http_server.bc
;;   ./target/release/lisp-vm /tmp/http_server.bc &
;;   curl http://localhost:8080/
;;   kill %1

(print "Starting HTTP server on port 8080...")
(def listener (http-listen 8080))
(print "Server ready! Waiting for connections...")

;; Handle 5 requests then exit
(loop ((count 0))
  (if (>= count 5)
    (print "Done!")
    (let ((stream (http-accept listener)))
      (let ((x (print "Connection received!")))
        (let ((request (http-read-request stream)))
          (let ((response (hash-map "status" 200 "body" "Hello from Lisp!")))
            (let ((y (http-send-response stream response)))
              (let ((z (http-close stream)))
                (recur (+ count 1))))))))))
