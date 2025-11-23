(defmacro when (cond body)
  `(if ,cond ,body false))

(defun positive? (x)
  (> x 0))

(defun negative? (x)
  (< x 0))

(defun check-number (n)
  (when (positive? n)
    (print "positive")))

(check-number 5)
(check-number -5)

(defun apply-twice (f x)
  ((lambda (y) (f (f y))) x))

(defmacro double (x)
  `(* ,x 2))

(print (apply-twice (lambda (n) (double n)) 3))

(defun map (f lst)
  (if (== lst '())
    '()
    (cons (f (car lst)) (map f (cdr lst)))))

(defmacro square (x)
  `(* ,x ,x))

(print (map (lambda (n) (square n)) '(1 2 3 4)))
