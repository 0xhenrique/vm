; Test environment frame construction

(defun make-env ()
  '())

(defun env-push-frame (params locals env)
  (cons (cons params (cons locals '())) env))

(defun env-current-params (env)
  (if (== env '())
    '()
    (car (car env))))

(defun env-current-locals (env)
  (if (== env '())
    '()
    (car (cdr (car env)))))

(print "=== Testing Frame Construction ===")
(print "")

(print "Test 1: Empty environment")
(print (make-env))

(print "")
(print "Test 2: Push frame with params")
(let ((params (cons "x" (cons "y" '()))))
  (print (env-push-frame params '() (make-env))))

(print "")
(print "Test 3: Extract params from frame")
(let ((params (cons "x" (cons "y" '()))))
  (print (env-current-params (env-push-frame params '() (make-env)))))

(print "")
(print "Test 4: Frame with locals")
(let ((locals (cons "temp" '())))
  (print (env-push-frame '() locals (make-env))))

(print "")
(print "Test 5: Extract locals from frame")
(let ((locals (cons "temp" '())))
  (print (env-current-locals (env-push-frame '() locals (make-env)))))
