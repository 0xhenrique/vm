;; HTTP Server with Routing Example
;; Demonstrates Phase 14 HTTP functionality with path-based routing
;;
;; Usage:
;;   ./target/release/bytecomp examples/http_server.lisp -o /tmp/http_server.bc
;;   ./target/release/lisp-vm /tmp/http_server.bc
;;
;; Then in another terminal:
;;   curl http://localhost:8080/
;;   curl http://localhost:8080/hello
;;   curl http://localhost:8080/test

(defun handle-request
  ((req)
    (let ((path (hashmap-get req "path")))
      (let ((method (hashmap-get req "method")))
        (let ((body (cond
                      ((== path "/") "Hello from Lisp HTTP Server!")
                      ((== path "/hello") "Hello, World!")
                      ((== path "/test") "This is a test response")
                      (true (string-append "404: Path not found: " path)))))
          (let ((status (cond
                          ((== path "/") 200)
                          ((== path "/hello") 200)
                          ((== path "/test") 200)
                          (true 404))))
            (hash-map "status" status "body" body)))))))

(defun serve
  ((port max-requests)
    (let ((listener (http-listen port)))
      (let ((msg (string-append "HTTP server listening on port " (number->string port))))
        (let ((x (print msg)))
          (loop ((count 0))
            (if (>= count max-requests)
              (print "Max requests reached, shutting down...")
              (let ((stream (http-accept listener)))
                (let ((request (http-read-request stream)))
                  (let ((response (handle-request request)))
                    (let ((y (http-send-response stream response)))
                      (let ((z (http-close stream)))
                        (recur (+ count 1))))))))))))))

;; Start server on port 8080, handle 10 requests then exit
(serve 8080 10)
