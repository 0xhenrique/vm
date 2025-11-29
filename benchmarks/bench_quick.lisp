;; Quick Benchmark - Reduced sizes to complete in reasonable time
;; Use this to get baseline numbers before optimization

(print "=== Quick Benchmark Suite ===")
(print "")

;; Helper
(defun make-list (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons i acc)))))

;; ============================================================================
;; 1. Small List Operations (100 elements)
;; ============================================================================
(print "1. List ops on 100 elements")
(def small-list (make-list 100))

(print "  map...")
(def r1 (map (lambda (x) (* x 2)) small-list))
(print (string-append "  map done, length: " (number->string (length r1))))

(print "  filter...")
(def r2 (filter even? small-list))
(print (string-append "  filter done, length: " (number->string (length r2))))

(print "  reduce...")
(def r3 (reduce + 0 small-list))
(print (string-append "  reduce done, sum: " (number->string r3)))
(print "")

;; ============================================================================
;; 2. Medium List Operations (1000 elements)
;; ============================================================================
(print "2. List ops on 1000 elements")
(def medium-list (make-list 1000))

(print "  map...")
(def m1 (map (lambda (x) (* x 2)) medium-list))
(print (string-append "  map done, length: " (number->string (length m1))))

(print "  filter...")
(def m2 (filter even? medium-list))
(print (string-append "  filter done, length: " (number->string (length m2))))

(print "  reduce...")
(def m3 (reduce + 0 medium-list))
(print (string-append "  reduce done, sum: " (number->string m3)))
(print "")

;; ============================================================================
;; 3. Arithmetic (no lists - baseline)
;; ============================================================================
(print "3. Pure arithmetic (100K iterations)")
(def arith-result
  (loop ((i 0) (acc 0))
    (if (>= i 100000)
        acc
        (recur (+ i 1) (+ acc (* i 2))))))
(print (string-append "  result: " (number->string arith-result)))
(print "")

;; ============================================================================
;; 4. Closure creation (1000 closures)
;; ============================================================================
(print "4. Create 1000 closures")
(defun make-adder (n) (lambda (x) (+ x n)))
(def closures
  (loop ((i 0) (acc '()))
    (if (>= i 1000)
        acc
        (recur (+ i 1) (cons (make-adder i) acc)))))
(print (string-append "  created: " (number->string (length closures))))
(print "")

;; ============================================================================
;; 5. Recursion vs loop/recur
;; ============================================================================
(print "5. Fibonacci")
(defun fib-iter (n)
  (loop ((i n) (a 0) (b 1))
    (if (<= i 0) a (recur (- i 1) b (+ a b)))))

(print (string-append "  fib(30): " (number->string (fib-iter 30))))
(print (string-append "  fib(40): " (number->string (fib-iter 40))))
(print "")

;; ============================================================================
;; 6. Cons vs Append (small scale)
;; ============================================================================
(print "6. Cons vs Append (building 500 element list)")

(print "  cons...")
(def cons-list
  (loop ((i 0) (acc '()))
    (if (>= i 500)
        acc
        (recur (+ i 1) (cons i acc)))))
(print (string-append "  cons done, length: " (number->string (length cons-list))))

(print "  append...")
(def append-list
  (loop ((i 0) (acc '()))
    (if (>= i 500)
        acc
        (recur (+ i 1) (append acc (list i))))))
(print (string-append "  append done, length: " (number->string (length append-list))))
(print "")

(print "=== Quick Benchmark Complete ===")
