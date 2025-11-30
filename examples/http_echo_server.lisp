;; HTTP Echo Server - Returns request details back to client
;; Tests header parsing and request introspection
;;
;; Usage:
;;   ./target/release/bytecomp examples/http_echo_server.lisp -o /tmp/http_echo.bc
;;   ./target/release/lisp-vm /tmp/http_echo.bc
;;
;; Test with:
;;   curl -H "X-Custom-Header: test" http://localhost:8080/

(defun echo-request
  ((req)
    (let ((method (hashmap-get req "method")))
      (let ((path (hashmap-get req "path")))
        (let ((method-part (string-append "Method: " method)))
          (let ((path-part (string-append "Path: " path)))
            (let ((body-text (string-append method-part path-part)))
              (hash-map "status" 200 "body" body-text))))))))

(defun run-echo-server
  ((port)
    (let ((listener (http-listen port)))
      (let ((x (print "Echo server listening on port 8080")))
        (loop ((count 0))
          (if (>= count 5)
            (print "Done!")
            (let ((stream (http-accept listener)))
              (let ((request (http-read-request stream)))
                (let ((response (echo-request request)))
                  (let ((y (http-send-response stream response)))
                    (let ((z (http-close stream)))
                      (recur (+ count 1)))))))))))))

(run-echo-server 8080)
