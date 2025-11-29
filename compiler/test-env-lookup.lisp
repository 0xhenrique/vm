; Test environment lookup functions

; === HELPERS ===

; Create empty environment
(defun make-env ()
  '())

; Create a new frame and push it onto environment
(defun env-push-frame (params locals env)
  (cons (cons params (cons locals '())) env))

; Get current frame's parameters
(defun env-current-params (env)
  (if (== env '())
    '()
    (car (car env))))

; Get current frame's locals
(defun env-current-locals (env)
  (if (== env '())
    '()
    (car (cdr (car env)))))

; Find index of name in list
(defun find-index-helper (name lst index)
  (if (== lst '())
    -1
    (if (== (car lst) name)
      index
      (find-index-helper name (cdr lst) (+ index 1)))))

(defun find-index (name lst)
  (find-index-helper name lst 0))

; Lookup variable in environment
(defun env-lookup (name env)
  (if (== env '())
    (cons "LoadGlobal" (cons name '()))
    (let ((params (env-current-params env)))
      (let ((locals (env-current-locals env)))
        (let ((param-idx (find-index name params)))
          (if (>= param-idx 0)
            (cons "LoadArg" (cons param-idx '()))
            (let ((local-idx (find-index name locals)))
              (if (>= local-idx 0)
                (cons "GetLocal" (cons local-idx '()))
                (cons "LoadGlobal" (cons name '()))))))))))

; === TESTS ===

(print "=== Environment Lookup Tests ===")
(print "")

(print "Test 1: Empty environment - expect LoadGlobal")
(print (env-lookup "x" (make-env)))

(print "")
(print "Test 2: Lookup x in params (x y) - expect LoadArg 0")
(print (env-lookup "x" (env-push-frame '("x" "y") '() (make-env))))

(print "")
(print "Test 3: Lookup y in params (x y) - expect LoadArg 1")
(print (env-lookup "y" (env-push-frame '("x" "y") '() (make-env))))

(print "")
(print "Test 4: Lookup z (not in env) - expect LoadGlobal")
(print (env-lookup "z" (env-push-frame '("x" "y") '() (make-env))))

(print "")
(print "Test 5: Lookup temp in locals - expect GetLocal 0")
(print (env-lookup "temp" (env-push-frame '("x") '("temp") (make-env))))

(print "")
(print "Test 6: Lookup x (param) vs temp (local)")
(print (env-lookup "x" (env-push-frame '("x") '("temp") (make-env))))
(print (env-lookup "temp" (env-push-frame '("x") '("temp") (make-env))))

(print "")
(print "Test 7: Multiple locals")
(print (env-lookup "a" (env-push-frame '() '("a" "b" "c") (make-env))))
(print (env-lookup "b" (env-push-frame '() '("a" "b" "c") (make-env))))
(print (env-lookup "c" (env-push-frame '() '("a" "b" "c") (make-env))))
