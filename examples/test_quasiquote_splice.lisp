(print `(a ,@'(1 2 3) b))

(print `(,@'(1 2 3)))

(print `(start ,@'() end))

(print `(a ,@'(b c) d ,@'(e f) g))

(defun get-list () '(x y z))
(print `(before ,@(get-list) after))

(print `(,@'(1) ,@'(2) ,@'(3)))
