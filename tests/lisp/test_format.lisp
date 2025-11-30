; Test string formatting with the format function
; format takes a format string with {} placeholders and a list of values

(print "=== Basic String Formatting ===")

; Simple string substitution
(print (format "Hello, {}!" (list "world")))
; => Hello, world!

; Multiple placeholders
(print (format "x={}, y={}" (list 10 20)))
; => x=10, y=20

; No placeholders
(print (format "Just a plain string" (list)))
; => Just a plain string

(print "")
(print "=== Different Types ===")

; Numbers
(print (format "The answer is {}" (list 42)))
; => The answer is 42

; Floats
(print (format "Pi is approximately {}" (list 3.14159)))
; => Pi is approximately 3.14159

; Booleans
(print (format "Success: {}" (list true)))
; => Success: true

; Mixed types
(print (format "Name: {}, Age: {}, Active: {}" (list "Alice" 30 true)))
; => Name: Alice, Age: 30, Active: true

(print "")
(print "=== Complex Values ===")

; Lists
(print (format "Numbers: {}" (list (list 1 2 3 4 5))))
; => Numbers: (1 2 3 4 5)

; Vectors
(print (format "Vector: {}" (list (vector 10 20 30))))
; => Vector: [10 20 30]

; Computations
(print (format "Sum: {}, Product: {}" (list (+ 5 7) (* 3 4))))
; => Sum: 12, Product: 12

(print "")
(print "=== Practical Examples ===")

; Logging
(defun log-message (level msg)
  (format "[{}] {}" (list level msg)))

(print (log-message "INFO" "Application started"))
(print (log-message "ERROR" "Connection failed"))

; User greeting
(defun greet (name age city)
  (format "Hello, {}! You are {} years old and live in {}." (list name age city)))

(print (greet "Bob" 25 "New York"))

; Data formatting
(defun format-user (user)
  (format "User(id={}, name={})" (list (car user) (car (cdr user)))))

(def user1 (list 1 "Alice"))
(print (format-user user1))

; Building SQL queries (demonstration only)
(defun build-query (table column value)
  (format "SELECT * FROM {} WHERE {} = {}" (list table column value)))

(print (build-query "users" "id" 42))

(print "")
(print "=== Nested Formatting ===")

; Format within format
(print (format "Outer: {}" (list (format "Inner: {}" (list "value")))))
; => Outer: Inner: value

; Building complex messages
(defun error-msg (func-name line-num msg)
  (format "Error in {}:{} - {}" (list func-name line-num msg)))

(print (error-msg "calculate" 42 "Division by zero"))

(print "")
(print "=== Done ===")
