(defun make-env ()
  '((bytecode . ())
    (functions . ())
    (param-names . ())
    (local-bindings . ())))

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

(defun env-append (env key new-items)
  (let ((current (env-get env key)))
    (env-set env key (append current new-items))))

(defun env-prepend (env key new-items)
  (let ((current (env-get env key)))
    (env-set env key (append new-items current))))

(defun integer? (x)
  (if (list? x)
    false
    (if (symbol? x)
      false
      (if (string? x)
        false
        (if (== x true)
          false
          (if (== x false)
            false
            true))))))

(defun boolean? (x)
  (if (== x true)
    true
    (if (== x false)
      true
      false)))

(defun index-of-helper
  ((elem '() idx) '())
  ((elem (h . t) idx)
    (if (== elem h)
      idx
      (index-of-helper elem t (+ idx 1)))))

(defun index-of (elem lst)
  (index-of-helper elem lst 0))

(defun emit (env instr)
  (env-append env 'bytecode (list instr)))

(defun emit-all (env instrs)
  (env-append env 'bytecode instrs))

(defun compile-expr (expr env)
  (if (integer? expr)
    (cons env (list (list 'push expr)))
    (if (boolean? expr)
      (cons env (list (list 'push expr)))
      (if (symbol? expr)
        (compile-variable expr env)
        (if (list? expr)
          (if (== expr '())
            (cons env (list (list 'error "Empty list")))
            (compile-list expr env))
          (cons env (list (list 'error "Unknown expr type"))))))))

(defun compile-variable (name env)
  (let ((params (env-get env 'param-names)))
    (let ((param-idx (index-of name params)))
      (if (== param-idx '())
        (cons env (list (list 'error "Undefined variable")))
        (cons env (list (list 'load-arg param-idx)))))))

(defun compile-list (lst env)
  (let ((first (car lst)))
    (if (symbol? first)
      (compile-operator first lst env)
      (cons env (list (list 'error "Expected operator"))))))

(defun compile-operator (op lst env)
  (match op
    ('+ (compile-plus lst env))
    ('- (compile-minus lst env))
    ('* (compile-mult lst env))
    ('/ (compile-div lst env))
    (_ (compile-call op lst env))))

(defun compile-plus (lst env)
  (let ((args (cdr lst)))
    (if (< (length args) 2)
      (cons env (list (list 'error "+ needs 2+ args")))
      (let ((result1 (compile-expr (car args) env)))
        (let ((env1 (car result1)))
          (let ((bc1 (cdr result1)))
            (compile-plus-rest (cdr args) env1 bc1)))))))

(defun compile-plus-rest
  (('() env bc) (cons env bc))
  (((h . t) env bc)
    (let* ((result (compile-expr h env))
           (env2 (car result))
           (bc2 (cdr result)))
      (compile-plus-rest t env2 (append bc (append bc2 '((add))))))))

(defun compile-minus (lst env)
  (let ((args (cdr lst)))
    (if (< (length args) 2)
      (cons env (list (list 'error "- needs 2+ args")))
      (let* ((result1 (compile-expr (car args) env))
             (env1 (car result1))
             (bc1 (cdr result1)))
        (compile-minus-rest (cdr args) env1 bc1)))))

(defun compile-minus-rest
  (('() env bc) (cons env bc))
  (((h . t) env bc)
    (let* ((result (compile-expr h env))
           (env2 (car result))
           (bc2 (cdr result)))
      (compile-minus-rest t env2 (append bc (append bc2 '((sub))))))))

(defun compile-mult (lst env)
  (let ((args (cdr lst)))
    (if (< (length args) 2)
      (cons env (list (list 'error "* needs 2+ args")))
      (let* ((result1 (compile-expr (car args) env))
             (env1 (car result1))
             (bc1 (cdr result1)))
        (compile-mult-rest (cdr args) env1 bc1)))))

(defun compile-mult-rest
  (('() env bc) (cons env bc))
  (((h . t) env bc)
    (let* ((result (compile-expr h env))
           (env2 (car result))
           (bc2 (cdr result)))
      (compile-mult-rest t env2 (append bc (append bc2 '((mul))))))))

(defun compile-div (lst env)
  (let ((args (cdr lst)))
    (if (< (length args) 2)
      (cons env (list (list 'error "/ needs 2+ args")))
      (let* ((result1 (compile-expr (car args) env))
             (env1 (car result1))
             (bc1 (cdr result1)))
        (compile-div-rest (cdr args) env1 bc1)))))

(defun compile-div-rest
  (('() env bc) (cons env bc))
  (((h . t) env bc)
    (let* ((result (compile-expr h env))
           (env2 (car result))
           (bc2 (cdr result)))
      (compile-div-rest t env2 (append bc (append bc2 '((div))))))))

(defun compile-call (func-name lst env)
  (let ((args (cdr lst)))
    (let* ((result (compile-args args env))
           (env2 (car result))
           (args-bc (cdr result))
           (argc (length args)))
      (cons env2 (append args-bc (list (list 'call func-name argc)))))))

(defun compile-args
  (('() env) (cons env '()))
  (((h . t) env)
    (let* ((result1 (compile-expr h env))
           (env1 (car result1))
           (bc1 (cdr result1))
           (result2 (compile-args t env1))
           (env2 (car result2))
           (bc2 (cdr result2)))
      (cons env2 (append bc1 bc2)))))

(print "Test 1: literal")
(let ((result (compile-expr 42 (make-env))))
  (print (cdr result)))

(print "Test 2: addition")
(let ((result (compile-expr '(+ 1 2) (make-env))))
  (print (cdr result)))

(print "Test 3: nested")
(let ((result (compile-expr '(+ 1 (* 2 3)) (make-env))))
  (print (cdr result)))

(print "Test 4: parameter")
(let* ((env (make-env))
       (env2 (env-set env 'param-names '(x y)))
       (result (compile-expr 'x env2)))
  (print (cdr result)))

(print "Test 5: function call")
(let ((result (compile-expr '(square 5) (make-env))))
  (print (cdr result)))
