; Test v16 features: cond, and, or, comparison operators

(print "Testing v16 features")
(print "")

; Test comparison operators
(print "Comparison operators:")
(print (< 3 5))        ; true
(print (< 5 3))        ; false
(print (> 5 3))        ; true
(print (> 3 5))        ; false
(print (<= 3 5))       ; true
(print (<= 5 5))       ; true
(print (<= 5 3))       ; false
(print (>= 5 3))       ; true
(print (>= 5 5))       ; true
(print (>= 3 5))       ; false
(print (!= 3 5))       ; true
(print (!= 5 5))       ; false
(print "")

; Test logical and
(print "Logical and:")
(print (and true true))            ; true
(print (and true false))           ; false
(print (and false true))           ; false
(print (and (< 3 5) (> 5 3)))     ; true
(print (and (< 3 5) (> 3 5)))     ; false
(print "")

; Test logical or
(print "Logical or:")
(print (or true true))             ; true
(print (or true false))            ; true
(print (or false true))            ; true
(print (or false false))           ; false
(print (or (< 3 5) (> 3 5)))      ; true
(print (or (> 3 5) (< 5 3)))      ; false
(print "")

; Test cond
(print "Cond expression:")
(defun classify-number (n)
  (cond
    ((< n 0) "negative")
    ((== n 0) "zero")
    ((> n 0) "positive")
    (else "unknown")))

(print (classify-number -5))      ; "negative"
(print (classify-number 0))       ; "zero"
(print (classify-number 5))       ; "positive"
(print "")

; Test abs using cond
(defun abs-v16 (n)
  (cond
    ((< n 0) (- 0 n))
    (else n)))

(print "Abs using cond:")
(print (abs-v16 -10))             ; 10
(print (abs-v16 10))              ; 10
(print (abs-v16 0))               ; 0
(print "")

; Test sign function using cond
(defun sign (n)
  (cond
    ((< n 0) -1)
    ((> n 0) 1)
    (else 0)))

(print "Sign function:")
(print (sign -5))                 ; -1
(print (sign 5))                  ; 1
(print (sign 0))                  ; 0
(print "")

; Test complex logical expression
(defun in-range (x low high)
  (and (>= x low) (<= x high)))

(print "In-range function:")
(print (in-range 5 0 10))         ; true
(print (in-range -1 0 10))        ; false
(print (in-range 11 0 10))        ; false
(print "")

(print "All v16 tests complete!")
