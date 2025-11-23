(defun max (a b)
  (if (> a b) a b))

(defun min (a b)
  (if (< a b) a b))

(defun clamp (x low high)
  (min (max x low) high))

(print (max 10 5))
(print (min 10 5))
(print (clamp 15 0 10))
(print (clamp -5 0 10))
(print (clamp 5 0 10))
