;; Benchmark: List Operations
;; Tests: map, filter, reduce, append, cons, reverse
;; This benchmark measures the performance of core list operations
;; which are suspected to have cloning overhead.

(print "=== Benchmark: List Operations ===")
(print "")

;; Configuration - adjust these for different intensity levels
(def LIST_SIZE 10000)
(def ITERATIONS 10)

;; Helper: Create a list of N numbers using loop/recur (efficient)
(defun make-list (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons i acc)))))

;; Helper: Check if a number is even
(defun even? (n) (= (% n 2) 0))

;; Helper: Time a single operation (returns result, prints nothing)
(defun run-n-times (n f)
  (loop ((i 0) (result '()))
    (if (>= i n)
        result
        (recur (+ i 1) (f)))))

;; ============================================================================
;; Benchmark 1: List Creation
;; ============================================================================
(print "Benchmark 1: List Creation")
(print (string-append (string-append "Creating list of " (number->string LIST_SIZE)) " elements..."))

(def test-list (make-list LIST_SIZE))
(print (string-append "List created. Length: " (number->string (length test-list))))
(print "")

;; ============================================================================
;; Benchmark 2: Map Operation
;; ============================================================================
(print "Benchmark 2: Map Operation")
(print (string-append (string-append "Running map " (number->string ITERATIONS)) " times..."))

(defun bench-map ()
  (map (lambda (x) (* x 2)) test-list))

(def map-result (run-n-times ITERATIONS bench-map))
(print (string-append "Map complete. Result length: " (number->string (length map-result))))
(print "")

;; ============================================================================
;; Benchmark 3: Filter Operation
;; ============================================================================
(print "Benchmark 3: Filter Operation")
(print (string-append (string-append "Running filter " (number->string ITERATIONS)) " times..."))

(defun bench-filter ()
  (filter even? test-list))

(def filter-result (run-n-times ITERATIONS bench-filter))
(print (string-append "Filter complete. Result length: " (number->string (length filter-result))))
(print "")

;; ============================================================================
;; Benchmark 4: Reduce Operation
;; ============================================================================
(print "Benchmark 4: Reduce Operation")
(print (string-append (string-append "Running reduce " (number->string ITERATIONS)) " times..."))

(defun bench-reduce ()
  (reduce + 0 test-list))

(def reduce-result (run-n-times ITERATIONS bench-reduce))
(print (string-append "Reduce complete. Sum: " (number->string reduce-result)))
(print "")

;; ============================================================================
;; Benchmark 5: Reverse Operation
;; ============================================================================
(print "Benchmark 5: Reverse Operation")
(print (string-append (string-append "Running reverse " (number->string ITERATIONS)) " times..."))

(defun bench-reverse ()
  (reverse test-list))

(def reverse-result (run-n-times ITERATIONS bench-reverse))
(print (string-append "Reverse complete. First element: " (number->string (car reverse-result))))
(print "")

;; ============================================================================
;; Benchmark 6: Append Operation (known to be expensive)
;; ============================================================================
(print "Benchmark 6: Append Operation")
(print "Building list via repeated append (expensive pattern)...")

(defun build-with-append (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (append acc (list i))))))

;; Use smaller size for append - it's O(n^2)
(def APPEND_SIZE 1000)
(def append-result (build-with-append APPEND_SIZE))
(print (string-append "Append complete. Length: " (number->string (length append-result))))
(print "")

;; ============================================================================
;; Benchmark 7: Cons Operation (should be fast)
;; ============================================================================
(print "Benchmark 7: Cons Operation")
(print "Building list via repeated cons (efficient pattern)...")

(defun build-with-cons (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons i acc)))))

(def cons-result (build-with-cons LIST_SIZE))
(print (string-append "Cons complete. Length: " (number->string (length cons-result))))
(print "")

;; ============================================================================
;; Benchmark 8: Chained Operations (map -> filter -> reduce)
;; ============================================================================
(print "Benchmark 8: Chained Operations")
(print "Running map -> filter -> reduce pipeline...")

(defun bench-pipeline ()
  (reduce + 0
    (filter even?
      (map (lambda (x) (* x 2)) test-list))))

(def pipeline-result (run-n-times ITERATIONS bench-pipeline))
(print (string-append "Pipeline complete. Result: " (number->string pipeline-result)))
(print "")

;; ============================================================================
;; Benchmark 9: Nested List Operations
;; ============================================================================
(print "Benchmark 9: Nested List Operations")
(print "Creating and processing nested lists...")

(defun make-nested-list (outer inner)
  (map (lambda (i) (make-list inner)) (make-list outer)))

(def NESTED_OUTER 100)
(def NESTED_INNER 100)
(def nested-list (make-nested-list NESTED_OUTER NESTED_INNER))
(print (string-append "Nested list created. Outer length: " (number->string (length nested-list))))

;; Flatten-like operation
(defun sum-nested (lst)
  (reduce + 0 (map (lambda (inner) (reduce + 0 inner)) lst)))

(def nested-sum (sum-nested nested-list))
(print (string-append "Nested sum: " (number->string nested-sum)))
(print "")

(print "=== List Operations Benchmark Complete ===")
