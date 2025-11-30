;; HTTP Status Code Test Server
;; Tests different HTTP status codes (200, 404, 500)
;;
;; Usage:
;;   ./target/release/bytecomp examples/http_status_codes.lisp -o /tmp/http_status.bc
;;   ./target/release/lisp-vm /tmp/http_status.bc
;;
;; Test with:
;;   curl -v http://localhost:8080/ok        # Returns 200
;;   curl -v http://localhost:8080/notfound  # Returns 404
;;   curl -v http://localhost:8080/error     # Returns 500

(defun handle-status-request
  ((req)
    (let ((path (hashmap-get req "path")))
      (cond
        ((== path "/ok")
          (hash-map "status" 200 "body" "OK - Status 200"))
        ((== path "/created")
          (hash-map "status" 201 "body" "Created - Status 201"))
        ((== path "/nocontent")
          (hash-map "status" 204 "body" ""))
        ((== path "/notfound")
          (hash-map "status" 404 "body" "Not Found - Status 404"))
        ((== path "/error")
          (hash-map "status" 500 "body" "Internal Server Error - Status 500"))
        (true
          (hash-map "status" 404 "body" "Unknown path"))))))

(defun run-status-server
  ((port)
    (let ((listener (http-listen port)))
      (let ((x (print "Status code test server listening on port 8080")))
        (loop ((count 0))
          (if (>= count 10)
            (print "Done!")
            (let ((stream (http-accept listener)))
              (let ((request (http-read-request stream)))
                (let ((response (handle-status-request request)))
                  (let ((y (http-send-response stream response)))
                    (let ((z (http-close stream)))
                      (recur (+ count 1)))))))))))))

(run-status-server 8080)
