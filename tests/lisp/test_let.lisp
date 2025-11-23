(print (let ((x 5)) (+ x 1)))

(print (let ((x 1) (y 2)) (+ x y)))

(print (let ((x 10) (y 20)) (* x y)))

(print (let ((x 5))
  (let ((y 3))
    (+ x y))))

(print (let ((x 1))
  (let ((x 2))
    x)))

(print (let (((h . t) '(1 2 3))) h))

(print (let (((h . t) '(1 2 3))) t))

(print (let (((a b c) '(10 20 30))) (+ a (+ b c))))

(print (let (((h . _) '(7 8 9))) h))

(defun test_let (n)
  (let ((x (* n 2)))
    (+ x 1)))

(print (test_let 5))

(print (let ((lst '(1 2 3)))
  (car lst)))

(print (let (((a b) '(5 10)))
  (let ((sum (+ a b)))
    (* sum 2))))

(print (let (((h1 . t1) '(1 2 3))
             ((h2 . t2) '(4 5 6)))
  (+ h1 h2)))
