(print `(a `(b ,(+ 1 2) c)))

(print `(outer ,(+ 5 5) (inner ,(+ 10 10))))

(print `((,1 ,2) (,3 (,4 ,5))))

(defun make-pair (a b) `(,a ,b))
(print (make-pair 1 2))
(print (make-pair 'x 'y))

(print `(list ,@(make-pair 10 20) end))
