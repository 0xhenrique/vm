(defmacro list1 (a)
  `(cons ,a '()))

(defmacro list2 (a b)
  `(cons ,a (cons ,b '())))

(defmacro list3 (a b c)
  `(cons ,a (cons ,b (cons ,c '()))))

(print (list1 1))
(print (list2 1 2))
(print (list3 1 2 3))

(print (list1 (+ 10 20)))
(print (list2 (* 2 3) (+ 4 5)))

(defmacro pair (a b)
  `'(,a ,b))

(print (pair 1 2))
(print (pair x y))
