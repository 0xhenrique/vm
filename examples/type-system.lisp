;; Phase 2: Type System Enhancements Demo
;; This example demonstrates the new type predicates and conversions

(defun demo-type-predicates ()
  (print "=== Type Predicates Demo ===")

  ;; Test integer?
  (print (if (integer? 42) "42 is an integer" "42 is not an integer"))
  (print (if (integer? "hello") "hello is an integer" "hello is not an integer"))

  ;; Test boolean?
  (print (if (boolean? true) "true is a boolean" "true is not a boolean"))
  (print (if (boolean? 1) "1 is a boolean" "1 is not a boolean"))

  ;; Test function?
  (print (if (function? +) "+ is a function" "+ is not a function"))

  ;; Test closure?
  (let ((my-closure (lambda (x) (+ x 1))))
    (print (if (closure? my-closure) "lambda is a closure" "lambda is not a closure"))
    (print (if (closure? +) "+ is a closure" "+ is not a closure")))

  ;; Test procedure? (either function or closure)
  (print (if (procedure? +) "+ is a procedure" "+ is not a procedure"))
  (let ((my-closure (lambda (x) (* x 2))))
    (print (if (procedure? my-closure) "lambda is a procedure" "lambda is not a procedure")))

  ;; Test number? (alias for integer? since we only have integers)
  (print (if (number? 123) "123 is a number" "123 is not a number")))

(defun demo-type-conversions ()
  (print "\n=== Type Conversions Demo ===")

  ;; string->number
  (let ((str "42")
        (num (string->number str)))
    (print (string-append "String '42' converted to number: " (number->string num)))
    (print (string-append "42 + 8 = " (number->string (+ num 8)))))

  ;; number->string (already existed, but shown for completeness)
  (let ((n 999))
    (print (string-append "Number 999 as string: '" (number->string n) "'")))

  ;; list->vector
  (let ((lst (list 1 2 3 4 5)))
    (let ((vec (list->vector lst)))
      (print "Converted list to vector")
      (print (string-append "Vector length: " (number->string (vector-length vec))))
      (print (string-append "Element at index 2: " (number->string (vector-ref vec 2))))))

  ;; vector->list
  (let ((vec (vector 10 20 30)))
    (let ((lst (vector->list vec)))
      (print "Converted vector to list")
      (print (string-append "List length: " (number->string (list-length lst))))
      (print (string-append "Second element: " (number->string (car (cdr lst)))))))

  ;; Roundtrip conversion
  (let ((original (list 7 8 9)))
    (let ((as-vec (list->vector original)))
      (let ((back-to-list (vector->list as-vec)))
        (print "Roundtrip: list -> vector -> list")
        (print (string-append "Final list length: " (number->string (list-length back-to-list))))))))

(defun type-system-demo ()
  (demo-type-predicates)
  (demo-type-conversions)
  (print "\n=== All Phase 2 features demonstrated! ==="))

;; Run the demo
(type-system-demo)
