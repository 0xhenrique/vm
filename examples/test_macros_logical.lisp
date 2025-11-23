(defmacro and (a b)
  `(if ,a ,b false))

(defmacro or (a b)
  `(if ,a true ,b))

(defmacro not (x)
  `(if ,x false true))

(print (and true true))
(print (and true false))
(print (and false true))
(print (and false false))

(print (or true true))
(print (or true false))
(print (or false true))
(print (or false false))

(print (not true))
(print (not false))

(print (and (> 5 3) (< 2 4)))
(print (or (> 5 10) (< 2 4)))
(print (not (> 5 10)))
