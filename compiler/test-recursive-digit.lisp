; Test is-digit in recursive context

(defun is-digit? (c)
  (let ((n (char-code c)))
    (>= n 48)))

(defun count-digits (chars n)
  (if (== chars '())
    n
    (if (is-digit? (car chars))
      (count-digits (cdr chars) (+ n 1))
      (count-digits (cdr chars) n))))

(print (count-digits (string->list "a1b2c3") 0))
