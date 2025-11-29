;; Benchmark: Recursion vs Iteration
;; Compares traditional recursion with loop/recur
;; Tests stack efficiency and performance differences

(print "=== Benchmark: Recursion vs Iteration ===")
(print "")

;; ============================================================================
;; Benchmark 1: Factorial - Recursive vs Iterative
;; ============================================================================
(print "Benchmark 1: Factorial")

;; Recursive factorial (limited by stack)
(defun factorial-recursive (n)
  (if (<= n 1)
      1
      (* n (factorial-recursive (- n 1)))))

;; Iterative factorial with loop/recur (no stack growth)
(defun factorial-iterative (n)
  (loop ((i n) (acc 1))
    (if (<= i 1)
        acc
        (recur (- i 1) (* acc i)))))

(print "Recursive factorial(20):")
(print (factorial-recursive 20))

(print "Iterative factorial(20):")
(print (factorial-iterative 20))
(print "")

;; ============================================================================
;; Benchmark 2: Fibonacci - Naive vs Memoized-style
;; ============================================================================
(print "Benchmark 2: Fibonacci")

;; Naive recursive fib - O(2^n), very slow
(defun fib-naive (n)
  (if (<= n 1)
      n
      (+ (fib-naive (- n 1)) (fib-naive (- n 2)))))

;; Iterative fib with loop/recur - O(n)
(defun fib-iterative (n)
  (loop ((i n) (a 0) (b 1))
    (if (<= i 0)
        a
        (recur (- i 1) b (+ a b)))))

(print "Naive fib(25) - this will be slow:")
(print (fib-naive 25))

(print "Iterative fib(25):")
(print (fib-iterative 25))

(print "Iterative fib(50) - only possible with loop/recur:")
(print (fib-iterative 50))
(print "")

;; ============================================================================
;; Benchmark 3: Sum of Range - Stack Depth Test
;; ============================================================================
(print "Benchmark 3: Sum of Range - Testing Stack Limits")

;; Recursive sum (will hit stack limit eventually)
(defun sum-recursive (n)
  (if (<= n 0)
      0
      (+ n (sum-recursive (- n 1)))))

;; Iterative sum (no stack limit)
(defun sum-iterative (n)
  (loop ((i n) (acc 0))
    (if (<= i 0)
        acc
        (recur (- i 1) (+ acc i)))))

(print "Recursive sum(1000):")
(print (sum-recursive 1000))

(print "Iterative sum(1000):")
(print (sum-iterative 1000))

(print "Iterative sum(100000) - not possible recursively:")
(print (sum-iterative 100000))
(print "")

;; ============================================================================
;; Benchmark 4: List Traversal
;; ============================================================================
(print "Benchmark 4: List Traversal")

;; Create test list
(defun make-list (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons i acc)))))

(def traverse-list (make-list 10000))

;; Recursive list length (will be slow/stack-limited)
(defun length-recursive (lst)
  (if (null? lst)
      0
      (+ 1 (length-recursive (cdr lst)))))

;; Iterative list length
(defun length-iterative (lst)
  (loop ((remaining lst) (count 0))
    (if (null? remaining)
        count
        (recur (cdr remaining) (+ count 1)))))

(print "List created with 10000 elements")

(print "Iterative length:")
(print (length-iterative traverse-list))
(print "")

;; ============================================================================
;; Benchmark 5: Mutual Recursion Pattern (using loop/recur workaround)
;; ============================================================================
(print "Benchmark 5: Even/Odd Check (Large Numbers)")

;; This demonstrates a pattern that would be mutual recursion
;; but implemented with loop/recur for efficiency
(defun is-even-iterative (n)
  (loop ((x (if (< n 0) (- 0 n) n)))
    (if (== x 0)
        true
        (if (== x 1)
            false
            (recur (- x 2))))))

(print "is-even-iterative(1000000):")
(print (is-even-iterative 1000000))

(print "is-even-iterative(1000001):")
(print (is-even-iterative 1000001))
(print "")

;; ============================================================================
;; Benchmark 6: Deep Recursion Stress Test
;; ============================================================================
(print "Benchmark 6: Deep Iteration Stress Test")

(defun count-to (n)
  (loop ((i 0))
    (if (>= i n)
        i
        (recur (+ i 1)))))

(print "Counting to 100000:")
(print (count-to 100000))

(print "Counting to 1000000:")
(print (count-to 1000000))
(print "")

;; ============================================================================
;; Benchmark 7: Ackermann-like (Highly Recursive)
;; ============================================================================
(print "Benchmark 7: Tak Function (Highly Recursive)")

;; Takeuchi function - classic recursion benchmark
(defun tak (x y z)
  (if (>= y x)
      z
      (tak (tak (- x 1) y z)
           (tak (- y 1) z x)
           (tak (- z 1) x y))))

(print "tak(18, 12, 6) - intensive recursion:")
(print (tak 18 12 6))
(print "")

(print "=== Recursion vs Iteration Benchmark Complete ===")
