; Debug tokenizer step by step

(defun is-digit? (c)
  (let ((n (char-code c)))
    (and (>= n 48) (<= n 57))))

(defun is-space? (c)
  (== (char-code c) 32))

(print "Step 1: string->list")
(print (string->list "(+ 1 2)"))

(print "Step 2: check first char")
(print (car (string->list "(+ 1 2)")))

(print "Step 3: check is-digit on +")
(print (is-digit? "+"))

(print "Step 4: check char-code of +")
(print (char-code "+"))
