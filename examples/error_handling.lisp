;; ============================================================
;; Error Handling Examples - Errors as Values
;; ============================================================
;;
;; This example demonstrates the error handling system based on
;; Rust/Go-style errors-as-values pattern.
;;
;; A Result is either:
;;   - (ok value)    - Success with a value
;;   - (err message) - Failure with an error message
;;
;; ============================================================

(load "stdlib.lisp")

;; ------------------------------------------------------------
;; Example 1: Basic Error Handling
;; ------------------------------------------------------------

(defun divide (x y)
  (if (== y 0)
      (err "division by zero")
      (ok (/ x y))))

(print "Example 1: Basic division with error checking")
(print (divide 10 2))       ;; => (ok 5)
(print (divide 10 0))       ;; => (err division by zero)
(print "")

;; ------------------------------------------------------------
;; Example 2: Unwrapping Values
;; ------------------------------------------------------------

(print "Example 2: Unwrapping successful results")
(let ((result (divide 20 4)))
  (if (ok? result)
      (print (unwrap result))
      (print (unwrap-err result))))
(print "")

;; ------------------------------------------------------------
;; Example 3: Default Values with unwrap-or
;; ------------------------------------------------------------

(print "Example 3: Providing default values")
(defun parse-number (s)
  (if (== s "42")
      (ok 42)
      (err "invalid number")))

(print (unwrap-or (parse-number "42") 0))      ;; => 42
(print (unwrap-or (parse-number "invalid") 0)) ;; => 0
(print "")

;; ------------------------------------------------------------
;; Example 4: Transforming Success Values with map-ok
;; ------------------------------------------------------------

(print "Example 4: Transforming ok values")
(defun double (x) (* x 2))

(print (map-ok double (ok 21)))              ;; => (ok 42)
(print (map-ok double (err "some error")))   ;; => (err some error)
(print "")

;; ------------------------------------------------------------
;; Example 5: Transforming Error Messages with map-err
;; ------------------------------------------------------------

(print "Example 5: Transforming error messages")
(defun add-context (msg)
  (string-append "Database error: " msg))

(print (map-err add-context (err "connection failed")))  ;; => (err Database error: connection failed)
(print (map-err add-context (ok 42)))                     ;; => (ok 42)
(print "")

;; ------------------------------------------------------------
;; Example 6: Chaining Operations with and-then
;; ------------------------------------------------------------

(print "Example 6: Chaining fallible operations")

(defun validate-positive (x)
  (if (> x 0)
      (ok x)
      (err "must be positive")))

(defun validate-even (x)
  (if (even? x)
      (ok x)
      (err "must be even")))

(defun safe-double (x)
  (ok (* x 2)))

;; Chain successful operations
(let ((result (and-then (validate-positive 4)
                        (lambda (x) (and-then (validate-even x)
                                             (lambda (y) (safe-double y)))))))
  (print result))  ;; => (ok 8)

;; Fail on first validation
(let ((result (and-then (validate-positive -4)
                        (lambda (x) (and-then (validate-even x)
                                             (lambda (y) (safe-double y)))))))
  (print result))  ;; => (err must be positive)

;; Fail on second validation
(let ((result (and-then (validate-positive 5)
                        (lambda (x) (and-then (validate-even x)
                                             (lambda (y) (safe-double y)))))))
  (print result))  ;; => (err must be even)

(print "")

;; ------------------------------------------------------------
;; Example 7: Error Recovery with or-else
;; ------------------------------------------------------------

(print "Example 7: Recovering from errors")

(defun try-parse-int (s)
  (if (== s "42")
      (ok 42)
      (err "parse error")))

(defun use-default (error-msg)
  (ok 0))

(print (unwrap (or-else (try-parse-int "42") use-default)))      ;; => 42
(print (unwrap (or-else (try-parse-int "invalid") use-default))) ;; => 0
(print "")

;; ------------------------------------------------------------
;; Example 8: Practical Use Case - Configuration Loading
;; ------------------------------------------------------------

(print "Example 8: Configuration loading with error handling")

(defun load-config (filename)
  (if (== filename "config.json")
      (ok '(8080 "localhost" "/api"))
      (err "file not found")))

(defun get-port (config) (car config))
(defun get-host (config) (car (cdr config)))
(defun get-path (config) (car (cdr (cdr config))))

;; Try loading config and extract port
(let ((config-result (load-config "config.json")))
  (if (ok? config-result)
      (print (get-port (unwrap config-result)))
      (print (unwrap-err config-result))))

;; Handle missing config file
(let ((config-result (load-config "missing.json")))
  (if (ok? config-result)
      (print "Config loaded")
      (print (unwrap-err config-result))))

(print "")

;; ------------------------------------------------------------
;; Example 9: Validation Pipeline
;; ------------------------------------------------------------

(print "Example 9: Multi-step validation pipeline")

(defun validate-range (min max x)
  (if (< x min)
      (err "value too small")
      (if (> x max)
          (err "value too large")
          (ok x))))

(defun validate-multiple-of (n x)
  (if (== (% x n) 0)
      (ok x)
      (err "value must be multiple of required number")))

(defun process-value (x)
  (ok (* x 10)))

;; Successful pipeline
(let ((result (and-then (validate-range 1 100 50)
                        (lambda (x) (and-then (validate-multiple-of 10 x)
                                             (lambda (y) (process-value y)))))))
  (if (ok? result)
      (print (unwrap result))
      (print (unwrap-err result))))

;; Fail range check
(let ((result (and-then (validate-range 1 100 150)
                        (lambda (x) (and-then (validate-multiple-of 10 x)
                                             (lambda (y) (process-value y)))))))
  (if (ok? result)
      (print (unwrap result))
      (print (unwrap-err result))))

;; Fail multiple check
(let ((result (and-then (validate-range 1 100 55)
                        (lambda (x) (and-then (validate-multiple-of 10 x)
                                             (lambda (y) (process-value y)))))))
  (if (ok? result)
      (print (unwrap result))
      (print (unwrap-err result))))

(print "")

;; ------------------------------------------------------------
;; Example 10: Result Predicates
;; ------------------------------------------------------------

(print "Example 10: Using result predicates")
(print (result? (ok 42)))    ;; => true
(print (result? (err "e")))  ;; => true
(print (result? 100))        ;; => false
(print (ok? (ok 42)))        ;; => true
(print (ok? (err "e")))      ;; => false
(print (err? (ok 42)))       ;; => false
(print (err? (err "e")))     ;; => true

(print "")
(print "All error handling examples completed!")
