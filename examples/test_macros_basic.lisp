(defmacro simple (x)
  x)

(print (simple 42))
(print (simple (+ 1 2)))

(defmacro quote-it (x)
  `',x)

(print (quote-it (+ 1 2)))

(defmacro const-5 ()
  5)

(print (const-5))

(defmacro add-one (x)
  `(+ ,x 1))

(print (add-one 10))
(print (add-one (+ 5 5)))
