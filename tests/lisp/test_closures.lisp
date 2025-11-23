(print ((lambda (x) (+ x 1)) 5))

(defun make-adder (n)
  (lambda (x) (+ x n)))

(print ((make-adder 10) 5))

(defun make-multiplier (a b)
  (lambda (x) (* x (+ a b))))

(print ((make-multiplier 2 3) 7))

(defun make-outer (x)
  (lambda (y)
    (lambda (z)
      (+ x (+ y z)))))

(print (((make-outer 1) 2) 3))

(print (let ((inc (lambda (x) (+ x 1))))
  (inc 42)))

(print (let ((n 100))
  ((lambda (x) (+ x n)) 23)))

(defun make-counter ()
  (let ((count 0))
    (lambda ()
      (let ((old count))
        old))))

(print ((make-counter)))

(defun apply-twice (f x)
  (f (f x)))

(print (apply-twice (lambda (n) (* n 2)) 3))

(defun curry-add (a)
  (lambda (b)
    (lambda (c)
      (+ a (+ b c)))))

(print (((curry-add 1) 2) 3))

(print ((lambda (x) x) 99))

(print ((lambda (lst) (car lst)) '(1 2 3)))

(print (let ((x 5))
  (let ((f (lambda (y) (+ x y))))
    (f 7))))

(print ((lambda (a b c) (+ a (+ b c))) 1 2 3))

(defun make-comparator (threshold)
  (lambda (x)
    (if (> x threshold)
      1
      0)))

(print ((make-comparator 10) 15))
(print ((make-comparator 10) 5))
