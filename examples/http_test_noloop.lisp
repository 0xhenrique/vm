;; HTTP test without loop - to debug the connection issue

(print "Starting HTTP server on port 8080...")
(def listener (http-listen 8080))
(print "Listening...")

;; Request 1
(print "Waiting for request 1...")
(def stream1 (http-accept listener))
(print "Got request 1")
(def req1 (http-read-request stream1))
(print "Read request 1, creating response...")
(def resp1 (hash-map "status" 200 "body" "Response 1"))
(print "Sending response 1...")
(http-send-response stream1 resp1)
(print "Closing stream 1...")
(http-close stream1)
(print "Closed request 1")

;; Request 2
(print "Waiting for request 2...")
(def stream2 (http-accept listener))
(print "Got request 2")
(def req2 (http-read-request stream2))
(def resp2 (hash-map "status" 200 "body" "Response 2"))
(http-send-response stream2 resp2)
(http-close stream2)
(print "Closed request 2")

(print "Done!")
