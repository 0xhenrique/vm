(defun make-env ()
  '((bytecode . ())
    (functions . ())
    (param-names . ())))

(defun env-get (env key)
  (cdr (assoc-lookup key env)))

(defun assoc-lookup
  ((key '()) '())
  ((key ((k . v) . rest))
    (if (== k key)
      (cons k v)
      (assoc-lookup key rest))))

(defun env-set (env key value)
  (cons (cons key value)
        (filter (lambda (pair) (not (== (car pair) key))) env)))

(defun compile-add (a b env)
  (let ((ra (compile-expr a env)))
    (let ((env1 (car ra)))
      (let ((bca (cdr ra)))
        (let ((rb (compile-expr b env1)))
          (let ((env2 (car rb)))
            (let ((bcb (cdr rb)))
              (cons env2 (append bca (append bcb '((add))))))))))))

(defun compile-expr-list (lst env)
  (let ((op (car lst)))
    (if (== op '+)
      (let ((args (cdr lst)))
        (compile-add (car args) (car (cdr args)) env))
      (cons env '((error "Unknown op"))))))

(defun compile-expr (expr env)
  (match expr
    (42 (cons env '((push 42))))
    (1 (cons env '((push 1))))
    (2 (cons env '((push 2))))
    (3 (cons env '((push 3))))
    (5 (cons env '((push 5))))
    (true (cons env '((push true))))
    (false (cons env '((push false))))
    ((h . t) (compile-expr-list expr env))
    (_ (cons env '((error "Unknown expr"))))))

(print "Test: 42")
(print (cdr (compile-expr 42 (make-env))))

(print "Test: (+ 1 2)")
(print (cdr (compile-expr '(+ 1 2) (make-env))))

(print "Test: (+ 5 (+ 2 3))")
(print (cdr (compile-expr '(+ 5 (+ 2 3)) (make-env))))
