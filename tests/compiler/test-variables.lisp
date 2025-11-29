(defun test-params
  ((x y) (+ x y)))

(print (test-params 10 20))

(print (let ((x 5) (y 3)) (+ x y)))

(print (let ((a 1))
  (let ((b (+ a 2)))
    (* b 3))))
