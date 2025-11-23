(print `,(+ 1 2))

(print `(1 ,(+ 2 3) 4))

(print `(a ,(+ 10 20) b))

(defun double (x) (* x 2))
(print `(result is ,(double 5)))

(print `(,(+ 1 1) ,(+ 2 2) ,(+ 3 3)))

(print `((,1 ,2) (,3 ,4)))
