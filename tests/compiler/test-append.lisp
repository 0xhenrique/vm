; Test: append-bytecode behavior

(defun append-bytecode
  (('() bc2) bc2)
  (((h . t) bc2) (cons h (append-bytecode t bc2))))

(defconst bc1 '((load-arg 0) (push 0) (eq) (jmp-if-false 6)))
(defconst bc2 '((push true)))
(defconst bc3 '((ret)))

(print "bc1:")
(print bc1)

(print "bc2:")
(print bc2)

(print "bc1 + bc2:")
(print (append-bytecode bc1 bc2))

(print "bc1 + bc2 + bc3:")
(print (append-bytecode bc1 (append-bytecode bc2 bc3)))

(print "")
(print "Now test the full pattern:")
(defconst result (append-bytecode bc1
                                   (append-bytecode bc2
                                                    (append-bytecode bc3
                                                                     '((push false) (ret))))))
(print "Full result:")
(print result)
