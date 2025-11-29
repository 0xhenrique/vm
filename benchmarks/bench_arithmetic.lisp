;; Benchmark: Arithmetic Performance
;; Tests raw computation speed without list/closure overhead

(print "=== Benchmark: Arithmetic Performance ===")
(print "")

;; ============================================================================
;; Benchmark 1: Integer Arithmetic
;; ============================================================================
(print "Benchmark 1: Integer Arithmetic")

(defun int-arithmetic (n)
  (loop ((i 0) (acc 0))
    (if (>= i n)
        acc
        (recur (+ i 1)
               (+ acc (- (* i 3) (/ i 2)))))))

(print "Running 100,000 integer operations...")
(def int-result (int-arithmetic 100000))
(print (string-append "Result: " (number->string int-result)))
(print "")

;; ============================================================================
;; Benchmark 2: Floating Point Arithmetic
;; ============================================================================
(print "Benchmark 2: Floating Point Arithmetic")

(defun float-arithmetic (n)
  (loop ((i 0) (acc 0.0))
    (if (>= i n)
        acc
        (recur (+ i 1)
               (+ acc (* (sin (/ i 1000.0)) (cos (/ i 1000.0))))))))

(print "Running 10,000 trig operations...")
(def float-result (float-arithmetic 10000))
(print "Result:")
(print float-result)
(print "")

;; ============================================================================
;; Benchmark 3: Comparison Operations
;; ============================================================================
(print "Benchmark 3: Comparison Operations")

(defun comparison-bench (n)
  (loop ((i 0) (true-count 0))
    (if (>= i n)
        true-count
        (recur (+ i 1)
               (if (and (> i 1000) (< i 9000) (even? i))
                   (+ true-count 1)
                   true-count)))))

(print "Running 100,000 comparisons...")
(def cmp-result (comparison-bench 100000))
(print (string-append "True count: " (number->string cmp-result)))
(print "")

;; ============================================================================
;; Benchmark 4: Modular Arithmetic (GCD lots of times)
;; ============================================================================
(print "Benchmark 4: GCD Computation")

(defun gcd (a b)
  (loop ((x a) (y b))
    (if (== y 0)
        x
        (recur y (% x y)))))

(defun gcd-bench (n)
  (loop ((i 1) (acc 0))
    (if (> i n)
        acc
        (recur (+ i 1) (+ acc (gcd (* i 17) (* i 13)))))))

(print "Computing 10,000 GCDs...")
(def gcd-result (gcd-bench 10000))
(print (string-append "Sum of GCDs: " (number->string gcd-result)))
(print "")

;; ============================================================================
;; Benchmark 5: Prime Checking
;; ============================================================================
(print "Benchmark 5: Prime Number Generation")

(defun is-prime (n)
  (if (<= n 1)
      false
      (loop ((i 2))
        (if (> (* i i) n)
            true
            (if (== (% n i) 0)
                false
                (recur (+ i 1)))))))

(defun count-primes (limit)
  (loop ((i 2) (count 0))
    (if (> i limit)
        count
        (recur (+ i 1)
               (if (is-prime i) (+ count 1) count)))))

(print "Counting primes up to 10,000...")
(def prime-count (count-primes 10000))
(print (string-append "Prime count: " (number->string prime-count)))
(print "")

;; ============================================================================
;; Benchmark 6: Collatz Conjecture
;; ============================================================================
(print "Benchmark 6: Collatz Sequence Lengths")

(defun collatz-length (n)
  (loop ((x n) (len 0))
    (if (== x 1)
        len
        (recur (if (even? x) (/ x 2) (+ (* x 3) 1))
               (+ len 1)))))

(defun sum-collatz-lengths (limit)
  (loop ((i 1) (acc 0))
    (if (> i limit)
        acc
        (recur (+ i 1) (+ acc (collatz-length i))))))

(print "Summing Collatz lengths for 1..10000...")
(def collatz-sum (sum-collatz-lengths 10000))
(print (string-append "Total length: " (number->string collatz-sum)))
(print "")

;; ============================================================================
;; Benchmark 7: Power Computation
;; ============================================================================
(print "Benchmark 7: Power Computation")

(defun fast-power (base exp)
  (loop ((b base) (e exp) (acc 1))
    (if (== e 0)
        acc
        (if (even? e)
            (recur (* b b) (/ e 2) acc)
            (recur b (- e 1) (* acc b))))))

(defun power-bench (n)
  (loop ((i 1) (acc 0))
    (if (> i n)
        acc
        (recur (+ i 1) (+ acc (fast-power 2 20))))))

(print "Computing 2^20 a thousand times...")
(def power-result (power-bench 1000))
(print (string-append "Sum: " (number->string power-result)))
(print "")

;; ============================================================================
;; Benchmark 8: Monte Carlo Pi Estimation
;; ============================================================================
(print "Benchmark 8: Monte Carlo Pi Estimation")

;; Seed the random number generator for reproducibility
(seed-random 42)

(defun estimate-pi (samples)
  (loop ((i 0) (inside 0))
    (if (>= i samples)
        (* 4.0 (/ inside samples))
        (let ((x (random))
              (y (random)))
          (recur (+ i 1)
                 (if (<= (+ (* x x) (* y y)) 1.0)
                     (+ inside 1)
                     inside))))))

(print "Estimating Pi with 100,000 samples...")
(def pi-estimate (estimate-pi 100000))
(print "Pi estimate:")
(print pi-estimate)
(print "")

;; ============================================================================
;; Benchmark 9: Numeric Integration (Trapezoidal)
;; ============================================================================
(print "Benchmark 9: Numeric Integration")

(defun integrate (f a b n)
  (let ((h (/ (- b a) n)))
    (loop ((i 0) (sum 0.0))
      (if (> i n)
          (* h sum)
          (let ((x (+ a (* i h)))
                (weight (if (or (== i 0) (== i n)) 0.5 1.0)))
            (recur (+ i 1) (+ sum (* weight (f x)))))))))

;; Integrate sin(x) from 0 to pi (should be ~2)
(print "Integrating sin(x) from 0 to pi with 10000 intervals...")
(def integral-result (integrate (lambda (x) (sin x)) 0.0 3.14159265359 10000))
(print "Result (expected: 2.0):")
(print integral-result)
(print "")

;; ============================================================================
;; Benchmark 10: Matrix-like Operations (using lists)
;; ============================================================================
(print "Benchmark 10: Dot Product")

(defun make-vector (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons (random) acc)))))

(defun dot-product (v1 v2)
  (loop ((a v1) (b v2) (acc 0.0))
    (if (or (null? a) (null? b))
        acc
        (recur (cdr a) (cdr b) (+ acc (* (car a) (car b)))))))

(seed-random 123)
(def vec1 (make-vector 10000))
(def vec2 (make-vector 10000))

(print "Computing dot product of two 10,000-element vectors...")
(def dot-result (dot-product vec1 vec2))
(print "Dot product:")
(print dot-result)
(print "")

(print "=== Arithmetic Performance Benchmark Complete ===")
