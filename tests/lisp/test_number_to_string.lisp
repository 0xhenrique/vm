; Comprehensive tests for number->string primitive

(print "=== number->string Basic Tests ===")

; Test 1: Basic positive integers
(print "Test 1: Positive integers")
(print (number->string 0))      ; "0"
(print (number->string 1))      ; "1"
(print (number->string 42))     ; "42"
(print (number->string 100))    ; "100"
(print (number->string 999))    ; "999"

; Test 2: Negative integers
(print "")
(print "Test 2: Negative integers")
(print (number->string -1))     ; "-1"
(print (number->string -42))    ; "-42"
(print (number->string -100))   ; "-100"

; Test 3: Large numbers
(print "")
(print "Test 3: Large numbers")
(print (number->string 12345))     ; "12345"
(print (number->string 1000000))   ; "1000000"
(print (number->string -999999))   ; "-999999"

; Test 4: Using in string operations
(print "")
(print "Test 4: String concatenation with numbers")
(print (string-append "Number: " (number->string 42)))
; "Number: 42"
(print (string-append "Count: " (number->string 100)))
; "Count: 100"

; Test 5: Building messages
(print "")
(print "Test 5: Building messages")
(defun format-count (n)
  (string-append "Total items: " (number->string n)))

(print (format-count 5))     ; "Total items: 5"
(print (format-count 1000))  ; "Total items: 1000"

; Test 6: Multiple number conversions
(print "")
(print "Test 6: Multiple conversions")
(defun format-range (start end)
  (string-append
    (string-append (number->string start) " to ")
    (number->string end)))

(print (format-range 1 10))     ; "1 to 10"
(print (format-range 50 100))   ; "50 to 100"

; Test 7: Using with arithmetic
(print "")
(print "Test 7: Arithmetic results to strings")
(print (number->string (+ 10 20)))   ; "30"
(print (number->string (* 5 5)))     ; "25"
(print (number->string (- 100 42)))  ; "58"

; Test 8: Converting computed values
(print "")
(print "Test 8: Converting computed values")
(defun sum-to-string (a b)
  (number->string (+ a b)))

(print (sum-to-string 10 20))  ; "30"
(print (sum-to-string 5 -3))   ; "2"

; Test 9: List lengths as strings
(print "")
(print "Test 9: List lengths to strings")
(defun length-message (lst)
  (string-append "Length: " (number->string (list-length lst))))

(print (length-message '(1 2 3)))      ; "Length: 3"
(print (length-message '(a b c d e)))  ; "Length: 5"

; Test 10: Building numbered lists
(print "")
(print "Test 10: Numbered list items")
(defun format-item (num item)
  (string-append
    (string-append (number->string num) ". ")
    item))

(print (format-item 1 "First"))   ; "1. First"
(print (format-item 2 "Second"))  ; "2. Second"
(print (format-item 10 "Tenth"))  ; "10. Tenth"

; Test 11: Counter display
(print "")
(print "Test 11: Counter")
(defun count-up (n)
  (if (> n 3)
    (print "Done")
    (if true
      (print (string-append "Count: " (number->string n)))
      (count-up (+ n 1)))))

(count-up 0)
(count-up 1)
(count-up 2)
(count-up 3)
(count-up 4)

; Test 12: Error reporting with line numbers
(print "")
(print "Test 12: Error messages with numbers")
(defun format-error (line-num msg)
  (string-append
    (string-append "Line " (number->string line-num))
    (string-append ": " msg)))

(print (format-error 10 "syntax error"))
(print (format-error 42 "undefined variable"))

; Test 13: Using in conditionals
(print "")
(print "Test 13: Conditional number display")
(defun describe-number (n)
  (if (< n 0)
    (string-append "Negative: " (number->string n))
    (if (== n 0)
      "Zero"
      (string-append "Positive: " (number->string n)))))

(print (describe-number -5))   ; "Negative: -5"
(print (describe-number 0))    ; "Zero"
(print (describe-number 10))   ; "Positive: 10"

; Test 14: Converting list elements
(print "")
(print "Test 14: Converting list of numbers to strings")
(defun numbers-to-strings (nums)
  (if (== nums '())
    '()
    (cons (number->string (car nums))
          (numbers-to-strings (cdr nums)))))

(print (numbers-to-strings '(1 2 3)))     ; ("1" "2" "3")
(print (numbers-to-strings '(10 20 30)))  ; ("10" "20" "30")

; Test 15: String length as number and back
(print "")
(print "Test 15: String length round-trip")
(print (number->string (string-length "hello")))  ; "5"
(print (number->string (string-length "world")))  ; "5"

(print "")
(print "=== All number->string tests completed ===")
