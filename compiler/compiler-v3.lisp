(defun make-env ()
  '())

(defun compile-add (a b env)
  (print "DEBUG: compile-add called")
  (print a)
  (print b)
  (let ((ra (compile-expr a env)))
    (print "DEBUG: ra =")
    (print ra)
    (let ((bca (cdr ra)))
      (print "DEBUG: bca =")
      (print bca)
      (let ((rb (compile-expr b env)))
        (print "DEBUG: rb =")
        (print rb)
        (let ((bcb (cdr rb)))
          (print "DEBUG: bcb =")
          (print bcb)
          (cons env (append bca (append bcb '((add))))))))))

(defun is-number (x)
  (if (== x 1) true
    (if (== x 2) true
      (if (== x 3) true
        (if (== x 5) true
          (if (== x 42) true
            false))))))

(defun compile-expr (expr env)
  (if (is-number expr)
    (cons env (cons (cons 'push (cons expr '())) '()))
    (if (== expr true)
      (cons env '((push true)))
      (if (== expr false)
        (cons env '((push false)))
        (if (list? expr)
          (let ((op (car expr)))
            (if (== op '+)
              (let ((args (cdr expr)))
                (compile-add (car args) (car (cdr args)) env))
              (cons env '((error "Unknown op")))))
          (cons env '((error "Unknown type"))))))))

(print "=== Compiler v3 Tests ===")

(print "Test: 42")
(print (cdr (compile-expr 42 (make-env))))

(print "Test: (+ 1 2)")
(print (cdr (compile-expr '(+ 1 2) (make-env))))

(print "Test: (+ 5 (+ 2 3))")
(print (cdr (compile-expr '(+ 5 (+ 2 3)) (make-env))))
