;; Phase 14: HTTP/Networking - Basic Tests
;; Tests the low-level HTTP API (http-listen, http-accept, http-read-request, http-send-response, http-close)
;;
;; Note: This test only tests basic listener creation, not full HTTP flow.
;; See examples/http_server.lisp for a working example.

;; Test 1: http-listen creates a listener
(def listener (http-listen 9999))
(print (type-of listener))  ;; Should print: tcp-listener

;; Test 2: Type checking
(def is-listener (== (type-of listener) 'tcp-listener))
(print is-listener)  ;; Should print: true

;; Note: http-accept would block here waiting for a connection, so we can't test
;; the full flow without a client. See the examples directory for complete servers.

true
