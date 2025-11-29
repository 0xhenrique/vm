; Debug environment construction

(defun make-env ()
  '())

(defun env-push-frame (params locals env)
  (cons (cons params (cons locals '())) env))

; Test basic construction
(print "Test: Building environment frame")
(print "Empty env:")
(print (make-env))

(print "")
(print "Frame with params using cons:")
(let ((params (cons "x" (cons "y" '()))))
  (let ((frame (env-push-frame params '() (make-env))))
    (print frame)))

(print "")
(print "Extract params from frame:")
(let ((params (cons "x" (cons "y" '()))))
  (let ((frame (env-push-frame params '() (make-env))))
    (let ((extracted-params (car (car frame))))
      (print extracted-params))))

(print "")
(print "Test find-index:")

(defun find-index-helper (name lst index)
  (if (== lst '())
    -1
    (if (== (car lst) name)
      index
      (find-index-helper name (cdr lst) (+ index 1)))))

(defun find-index (name lst)
  (find-index-helper name lst 0))

(let ((params (cons "x" (cons "y" '()))))
  (print (find-index "x" params))
  (print (find-index "y" params))
  (print (find-index "z" params)))
