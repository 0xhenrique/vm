; Test functions returning complex structures

(defun read-stuff (chars)
  (cons "result" (cons chars '())))

(defun use-result (chars)
  (let ((res (read-stuff chars)))
    (car (cdr res))))

(print (use-result (string->list "hello")))
