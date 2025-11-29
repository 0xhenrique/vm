;; Benchmark: Closure Performance
;; Tests closure creation, invocation, and capture overhead

(print "=== Benchmark: Closure Performance ===")
(print "")

;; ============================================================================
;; Benchmark 1: Closure Creation
;; ============================================================================
(print "Benchmark 1: Mass Closure Creation")

;; Create many closures that capture a value
(defun make-adder (n)
  (lambda (x) (+ x n)))

(defun create-closures (count)
  (loop ((i 0) (acc '()))
    (if (>= i count)
        acc
        (recur (+ i 1) (cons (make-adder i) acc)))))

(def CLOSURE_COUNT 1000)
(print (string-append (string-append "Creating " (number->string CLOSURE_COUNT)) " closures..."))
(def closures (create-closures CLOSURE_COUNT))
(print (string-append (string-append "Created " (number->string (length closures))) " closures"))
(print "")

;; ============================================================================
;; Benchmark 2: Closure Invocation
;; ============================================================================
(print "Benchmark 2: Closure Invocation")

(defun invoke-n-times (n f arg)
  (loop ((i 0) (result 0))
    (if (>= i n)
        result
        (recur (+ i 1) (f arg)))))

(def INVOKE_COUNT 10000)
(print (string-append (string-append "Invoking closure " (number->string INVOKE_COUNT)) " times..."))
(def invoke-result (invoke-n-times INVOKE_COUNT (make-adder 10) 5))
(print (string-append "Result: " (number->string invoke-result)))
(print "")

;; ============================================================================
;; Benchmark 3: Closure with Multiple Captures
;; ============================================================================
(print "Benchmark 3: Multi-Capture Closures")

(defun make-linear (a b)
  (lambda (x) (+ (* a x) b)))

(print "f(x) = 3x + 5")
(print (string-append "f(10) = " (number->string ((make-linear 3 5) 10))))

;; Create many multi-capture closures
(defun create-linear-fns (count)
  (loop ((i 0) (acc '()))
    (if (>= i count)
        acc
        (recur (+ i 1) (cons (make-linear i (+ i 1)) acc)))))

(def linear-fns (create-linear-fns CLOSURE_COUNT))
(print (string-append (string-append "Created " (number->string (length linear-fns))) " linear functions"))
(print "")

;; ============================================================================
;; Benchmark 4: Nested Closures
;; ============================================================================
(print "Benchmark 4: Nested Closures")

;; Curried add
(defun curry-add (a)
  (lambda (b)
    (lambda (c) (+ a (+ b c)))))

(def result-1-2-3 (((curry-add 1) 2) 3))
(print (string-append "curry-add(1)(2)(3) = " (number->string result-1-2-3)))

;; Create nested closure chain - simplified version
(defun create-nested-adder (n)
  (if (<= n 0)
      (lambda (x) x)
      (let ((inner (create-nested-adder (- n 1))))
        (lambda (x) (inner (+ x 1))))))

(def NEST_DEPTH 50)
(print (string-append (string-append "Creating closure chain of depth " (number->string NEST_DEPTH)) "..."))
(print (string-append "Chain created. nested-fn(0) = " (number->string ((create-nested-adder NEST_DEPTH) 0))))
(print "")

;; ============================================================================
;; Benchmark 5: Closures with List Capture
;; ============================================================================
(print "Benchmark 5: Closures Capturing Lists")

(defun make-list (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons i acc)))))

;; Closure that captures a list
(defun make-list-summer (lst)
  (lambda () (reduce + 0 lst)))

(print "Closure capturing list of 1000 elements")
(print (string-append "Sum via closure: " (number->string ((make-list-summer (make-list 1000))))))
(print "")

;; ============================================================================
;; Benchmark 6: Higher-Order Function Chains
;; ============================================================================
(print "Benchmark 6: Higher-Order Function Chains")

;; compose creates closures - use let to enable calling
(print "compose(square, compose(double, inc))(5):")
(print (string-append "Result: "
  (number->string
    (let ((double-fn (lambda (x) (* x 2)))
          (inc-fn (lambda (x) (+ x 1)))
          (square-fn (lambda (x) (* x x))))
      (let ((composed-fn (compose square-fn (compose double-fn inc-fn))))
        (composed-fn 5))))))

;; Apply composed function many times
(defun apply-n-times (n f x)
  (loop ((i 0) (result x))
    (if (>= i n)
        result
        (recur (+ i 1) (f result)))))

(print "Applying inc 1000 times to 0:")
(defun my-inc (x) (+ x 1))
(print (string-append "Result: " (number->string (apply-n-times 1000 my-inc 0))))
(print "")

;; ============================================================================
;; Benchmark 7: Closure in Map/Filter/Reduce
;; ============================================================================
(print "Benchmark 7: Closures in HOFs")

(def data (make-list 5000))

;; map with closure
(defun bench-map-closure ()
  (let ((multiplier 7))
    (map (lambda (x) (* x multiplier)) data)))

(print "Map with capturing closure over 5000 elements:")
(def mapped (bench-map-closure))
(print (string-append "First 5: " (number->string (car mapped))))

;; filter with closure
(defun bench-filter-closure ()
  (let ((threshold 2500))
    (filter (lambda (x) (> x threshold)) data)))

(print "Filter with capturing closure:")
(def filtered (bench-filter-closure))
(print (string-append "Filtered count: " (number->string (length filtered))))
(print "")

;; ============================================================================
;; Benchmark 8: Closure Memory Pattern
;; ============================================================================
(print "Benchmark 8: Closure Memory Pattern")

;; Each closure captures its own copy of the list
(defun make-closure-with-list (size)
  (let ((lst (make-list size)))
    (lambda () (length lst))))

(defun create-memory-closures (count size)
  (loop ((i 0) (acc '()))
    (if (>= i count)
        acc
        (recur (+ i 1) (cons (make-closure-with-list size) acc)))))

(print "Creating 100 closures, each capturing a 100-element list...")
(def memory-closures (create-memory-closures 100 100))
(print (string-append (string-append "Created " (number->string (length memory-closures))) " closures"))

;; Invoke each
(def total-lengths (reduce + 0 (map (lambda (f) (f)) memory-closures)))
(print (string-append "Total lengths: " (number->string total-lengths)))
(print "")

(print "=== Closure Performance Benchmark Complete ===")
