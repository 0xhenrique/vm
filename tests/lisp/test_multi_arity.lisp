;; SKIP
;; Reason: Requires multi-clause function definitions (Phase 8)
(defun add
  ((x) x)
  ((x y) (+ x y))
  ((x y z) (+ x (+ y z))))

(print (add 5))
(print (add 5 3))
(print (add 5 3 2))

(defun greet
  (('default) "Hello, World")
  ((name) name))

(print (greet 'default))
(print (greet "Alice"))

(defun process
  ((0) "zero")
  ((1) "one")
  ((n) "many"))

(print (process 0))
(print (process 1))
(print (process 42))

(defun list-op
  (('()) "empty")
  (((x)) x)
  (((x y)) (+ x y))
  (((h . t)) h))

(print (list-op '()))
(print (list-op '(42)))
(print (list-op '(10 20)))
(print (list-op '(1 2 3 4)))

(defun maybe-add
  ((x 0) x)
  ((0 y) y)
  ((x y) (+ x y)))

(print (maybe-add 5 0))
(print (maybe-add 0 7))
(print (maybe-add 3 4))

(defun identity
  ((x) x))

(defun const
  ((x y) x))

(print (identity 100))
(print (const 42 99))

(defun first-or-default
  (('() default) default)
  (((h . t) default) h))

(print (first-or-default '() 0))
(print (first-or-default '(5 6 7) 0))

(defun nested-patterns
  (((0) (0)) "both zero")
  (((x) (y)) (+ x y)))

(print (nested-patterns '(0) '(0)))
(print (nested-patterns '(3) '(7)))

(defun complex-arity
  ((_) "one arg")
  ((_ _) "two args")
  ((_ _ _) "three args"))

(print (complex-arity 1))
(print (complex-arity 1 2))
(print (complex-arity 1 2 3))

(defun mixed-types
  ((0 'zero) "special")
  ((n 'zero) n)
  ((0 m) m)
  ((n m) (+ n m)))

(print (mixed-types 0 'zero))
(print (mixed-types 5 'zero))
(print (mixed-types 0 10))
(print (mixed-types 3 7))
