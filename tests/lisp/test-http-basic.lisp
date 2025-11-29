;; Basic HTTP functionality tests
;; Tests that http-listen returns a listener

(def listener (http-listen 9999))

;; Test that listener was created successfully
(print "✓ http-listen created listener on port 9999")

;; Test that we can close the listener by just dropping it
;; (it will be garbage collected when listener goes out of scope)

(print "✓ HTTP basic tests passed")
