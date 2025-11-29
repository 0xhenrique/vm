;; SKIP
;; Reason: Requires pattern matching and destructuring in function parameters (Phase 8)
(defun first ((h . _) h))
(defun rest ((_ . t) t))

(defun len
  (('()) 0)
  (((h . t)) (+ 1 (len t))))

(print (first '(1 2 3)))
(print (rest '(1 2 3)))
(print (len '()))
(print (len '(a b c)))

(defun fib
  ((0) 0)
  ((1) 1)
  ((n) (+ (fib (- n 1)) (fib (- n 2)))))

(print (fib 5))
