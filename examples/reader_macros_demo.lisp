; Reader Macros Demo
; Demonstrates the use of all built-in reader macros in the language
;
; Built-in Reader Macros:
; - #()   Vector literals
; - #t #f  Boolean literals (Scheme-style)
; - #;     Expression comments
; - #'     Function quote (visual clarity)

(print "=== Reader Macros Demo ===")
(print "")

; ===================================================================
; Built-in Reader Macro: #() for Vector Literals
; ===================================================================

(print "1. Basic vector literal:")
(def vec1 #(1 2 3 4 5))
(print vec1)
(print "")

(print "2. Empty vector:")
(def empty-vec #())
(print empty-vec)
(print "")

(print "3. Vector with expressions:")
(def vec2 #((+ 1 2) (* 3 4) (- 10 5)))
(print vec2)
(print "")

(print "4. Nested vectors:")
(def matrix #(#(1 2 3)
              #(4 5 6)
              #(7 8 9)))
(print matrix)
(print "")

(print "5. Vector with mixed types:")
(def mixed #(1 "hello" true 3.14 'symbol))
(print mixed)
(print "")

(print "6. Vector operations:")
(def nums1 #(10 20 30 40 50))
(print (vector-ref nums1 0))  ; First element
(print (vector-ref nums1 2))  ; Third element
(print (vector-length nums1)) ; Length
(print "")

(print "7. Vector with functions:")
(def funcs #((lambda (x) (* x 2))
             (lambda (x) (* x 3))
             (lambda (x) (* x 4))))
(print "Applying second function to 5:")
(print ((vector-ref funcs 1) 5))
(print "")

(print "8. Vectors in function arguments:")
(defun sum-vector-elements (v)
  (+ (vector-ref v 0)
     (vector-ref v 1)
     (vector-ref v 2)))

(print "Sum of #(10 20 30):")
(print (sum-vector-elements #(10 20 30)))
(print "")

(print "9. Comparison with explicit vector creation:")
(def v1 #(1 2 3))
(def v2 (vector 1 2 3))
(print "Both create the same vector:")
(print v1)
(print v2)
(print "")

(print "10. Processing vectors with higher-order functions:")
(def numbers #(1 2 3 4 5))
(print "Original vector:")
(print numbers)
(print "Accessing elements:")
(print (vector-ref numbers 0))
(print (vector-ref numbers 4))
(print "")

; ===================================================================
; Comparison with List Syntax
; ===================================================================

(print "11. Vectors vs Lists:")
(def my-list '(1 2 3))
(def my-vector #(1 2 3))
(print "List:")
(print my-list)
(print "Vector:")
(print my-vector)
(print "")

(print "12. Performance characteristics:")
(print "- Lists: Fast sequential access (car/cdr)")
(print "- Vectors: Fast random access (vector-ref)")
(print "")

; ===================================================================
; Advanced Examples
; ===================================================================

(print "13. 2D Matrix operations:")
(def matrix-2x3 #(#(1 2 3)
                   #(4 5 6)))

(defun matrix-get (m row col)
  (vector-ref (vector-ref m row) col))

(print "Element at (0, 2):")
(print (matrix-get matrix-2x3 0 2))
(print "Element at (1, 1):")
(print (matrix-get matrix-2x3 1 1))
(print "")

(print "14. Vector of quoted symbols:")
(def symbols #('foo 'bar 'baz))
(print symbols)
(print "")

(print "15. Deeply nested vectors:")
(def nested #(#(#(1 2) #(3 4))
               #(#(5 6) #(7 8))))
(print nested)
(print "")

; ===================================================================
; Built-in Reader Macro: #t and #f for Boolean Literals
; ===================================================================

(print "16. Boolean literals (Scheme-style):")
(print #t)    ; true
(print #f)    ; false
(print "")

(print "17. Booleans in conditionals:")
(if #t
    (print "This executes (true branch)")
    (print "This doesn't execute"))
(print "")

(print "18. Booleans in vectors:")
(def bool-vec #(#t #f true false))
(print bool-vec)
(print "")

; ===================================================================
; Built-in Reader Macro: #; for Expression Comments
; ===================================================================

(print "19. Expression comment - simple:")
(def result (+ 1 #;2 3))  ; 2 is commented out
(print result)  ; => 4
(print "")

(print "20. Expression comment - complex:")
(def result2 (+ 1 #;(* 10 20) 3 #;(- 100 50) 5))
(print result2)  ; => 9 (only 1, 3, and 5 are added)
(print "")

(print "21. Nested expression comments:")
(def result3 (+ 1 #;#;2 3 4))  ; Both 2 and 3 are commented
(print result3)  ; => 5
(print "")

(print "22. Commenting out function definitions:")
#;(defun bad-function (x)
    (/ x 0))  ; This function is never defined

(defun good-function (x)
  (* x 2))

(print (good-function 21))  ; => 42
(print "")

; ===================================================================
; Built-in Reader Macro: #' for Function Quote
; ===================================================================

(print "23. Function quote (visual clarity):")
(defun square (x) (* x x))
(#'square 7)  ; Same as (square 7), but visually indicates function reference
(print (#'square 7))
(print "")

(print "24. Function quote is equivalent to bare name:")
(print (square 7))   ; Same as above
(print (#'square 7)) ; Visually clearer intent
(print "")

(print "25. Function quote with builtins:")
(print (#'+ 10 32))  ; => 42
(print "")

; ===================================================================
; Combining Multiple Reader Macros
; ===================================================================

(print "26. Combining reader macros:")
(def data #(#t #f #;999 42 #;#;100 200 300))
(print data)  ; => #(true false 42 300)
(print "")

(print "27. Reader macros in function calls:")
(defun process (a b c)
  (+ a b c))

(print (#'process 10 #;20 20 #;30 12))  ; => 42
(print "")

(print "=== Demo Complete ===")
