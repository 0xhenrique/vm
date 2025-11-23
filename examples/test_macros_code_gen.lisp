(defmacro make-adder (n)
  `(lambda (x) (+ x ,n)))

(print ((make-adder 10) 5))

(defmacro swap-args (f a b)
  `(,f ,b ,a))

(print (swap-args - 10 3))
(print (swap-args / 20 4))

(defmacro repeat-twice (expr)
  `(if true ,expr ,expr))

(print (repeat-twice (+ 1 2)))
