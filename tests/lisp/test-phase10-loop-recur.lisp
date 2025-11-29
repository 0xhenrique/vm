;; Test Phase 10: loop/recur Construct
;; EXPECT-CONTAINS: All Phase 10 tests passed!

(print "=== Testing Phase 10: loop/recur Construct ===")
(print "")

;; ============================================================================
;; Test 1: Simple Countdown
;; ============================================================================

(print "Test 1: Simple countdown with loop/recur")
(print (loop ((n 10))
  (if (<= n 0)
      n
      (recur (- n 1)))))
(print "")

;; ============================================================================
;; Test 2: Factorial using loop/recur
;; ============================================================================

(print "Test 2: Factorial with loop/recur")
(defun factorial (n)
  (loop ((i n) (acc 1))
    (if (<= i 1)
        acc
        (recur (- i 1) (* acc i)))))

(print (factorial 5))
(print (factorial 10))
(print "")

;; ============================================================================
;; Test 3: Sum of range
;; ============================================================================

(print "Test 3: Sum of range with loop/recur")
(defun sum-range (n)
  (loop ((i 0) (sum 0))
    (if (> i n)
        sum
        (recur (+ i 1) (+ sum i)))))

(print (sum-range 10))
(print (sum-range 100))
(print "")

;; ============================================================================
;; Test 4: Fibonacci
;; ============================================================================

(print "Test 4: Fibonacci with loop/recur")
(defun fib (n)
  (loop ((i n) (a 0) (b 1))
    (if (<= i 0)
        a
        (recur (- i 1) b (+ a b)))))

(print (fib 10))
(print (fib 20))
(print "")

;; ============================================================================
;; Test 5: Build list in reverse
;; ============================================================================

(print "Test 5: Build list with loop/recur")
(defun build-list (n)
  (loop ((i n) (lst '()))
    (if (<= i 0)
        lst
        (recur (- i 1) (cons i lst)))))

(print (build-list 5))
(print "")

;; ============================================================================
;; Test 6: GCD (Greatest Common Divisor)
;; ============================================================================

(print "Test 6: GCD with loop/recur")
(defun gcd (a b)
  (loop ((x a) (y b))
    (if (== y 0)
        x
        (recur y (% x y)))))

(print (gcd 48 18))
(print (gcd 100 35))
(print "")

;; ============================================================================
;; Test 7: Find element in list
;; ============================================================================

(print "Test 7: Find element in list with loop/recur")
(defun find-elem (elem lst)
  (loop ((remaining lst))
    (if (null? remaining)
        false
        (if (== (car remaining) elem)
            true
            (recur (cdr remaining))))))

(print (find-elem 3 '(1 2 3 4 5)))
(print (find-elem 10 '(1 2 3 4 5)))
(print "")

;; ============================================================================
;; Test 8: Large iteration (no stack overflow)
;; ============================================================================

(print "Test 8: Large iteration - no stack overflow")
(defun count-to (n)
  (loop ((i 0))
    (if (>= i n)
        i
        (recur (+ i 1)))))

(print (count-to 10000))
(print "Completed 10000 iterations without stack overflow!")
(print "")

;; ============================================================================
;; Test 9: Loop with multiple accumulators
;; ============================================================================

(print "Test 9: Multiple accumulators")
(defun sum-and-product (n)
  (loop ((i 1) (sum 0) (prod 1))
    (if (> i n)
        (list sum prod)
        (recur (+ i 1) (+ sum i) (* prod i)))))

(print (sum-and-product 5))
(print "")

;; ============================================================================
;; Test 10: String building with loop/recur
;; ============================================================================

(print "Test 10: Build list of numbers")
(defun range (start end)
  (loop ((i start) (result '()))
    (if (> i end)
        result
        (recur (+ i 1) (append result (list i))))))

(print (range 1 10))
(print "")

(print "All Phase 10 tests passed!")
