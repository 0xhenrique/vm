(defmacro when (cond body)
  `(if ,cond ,body false))

(defmacro unless (cond body)
  `(if ,cond false ,body))

(defmacro and (a b)
  `(if ,a ,b false))

(defmacro or (a b)
  `(if ,a true ,b))

(print (when (> 10 5) (+ 1 2)))

(print (unless (< 10 5) (+ 3 4)))

(print (and true (> 5 3)))

(print (and false (> 5 3)))

(print (or true (> 5 3)))

(print (or false (> 5 3)))

(defun test-when (x)
  (when (> x 5) (print "big")))

(test-when 10)
(test-when 3)

(defmacro list2 (a b)
  `(cons ,a (cons ,b '())))

(print (list2 1 2))
