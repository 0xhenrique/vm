;; Performance Benchmarks for Phase 12a: Parallel Collections
;; Compare parallel vs sequential operations (when available)

(print "=== Parallel Collections Benchmarks ===")

;; Helper: Create a large list
(defun build-list-helper (n acc)
  (if (<= n 0)
      acc
      (build-list-helper (- n 1) (cons n acc))))

(defun build-list (n)
  (build-list-helper n '()))

;; Benchmark 1: pmap on computational task (expensive per-item operation)
(print "Benchmark 1: pmap with expensive computation")

;; Helper for expensive computation
(defun expensive-helper (x n acc)
  (if (<= n 0)
      acc
      (expensive-helper x (- n 1) (+ acc x))))

(defun expensive-square (x)
  (* x (expensive-helper x 1000 0)))  ;; Simulate expensive computation

(def test-data-small (build-list 100))
(print "  Dataset size: 100 items")
(def t1 (current-timestamp))
(def result1 (pmap expensive-square test-data-small))
(def t2 (current-timestamp))
(print (string-append "  pmap time: " (string-append (number->string (- t2 t1)) "ms")))
(print (string-append "  Result size: " (number->string (list-length result1))))

;; Benchmark 2: pfilter on predicate evaluation
(print "Benchmark 2: pfilter with predicate")
(defun is-interesting (x)
  (== (% x 7) 0))

(def test-data-medium (build-list 1000))
(print "  Dataset size: 1000 items")
(def t3 (current-timestamp))
(def result2 (pfilter is-interesting test-data-medium))
(def t4 (current-timestamp))
(print (string-append "  pfilter time: " (string-append (number->string (- t4 t3)) "ms")))
(print (string-append "  Filtered count: " (number->string (list-length result2))))

;; Benchmark 3: preduce for aggregation
(print "Benchmark 3: preduce for sum")
(defun add (a b) (+ a b))

(def test-data-large (build-list 10000))
(print "  Dataset size: 10000 items")
(def t5 (current-timestamp))
(def result3 (preduce test-data-large 0 add))
(def t6 (current-timestamp))
(print (string-append "  preduce time: " (string-append (number->string (- t6 t5)) "ms")))
(print (string-append "  Sum result: " (number->string result3)))

;; Benchmark 4: Combined pipeline
(print "Benchmark 4: Combined pmap + pfilter + preduce")
(def pipeline-data (build-list 500))
(print "  Dataset size: 500 items")

(def t7 (current-timestamp))
(def mapped (pmap (lambda (x) (* x x)) pipeline-data))
(def filtered (pfilter (lambda (x) (> x 100)) mapped))
(def reduced (preduce filtered 0 (lambda (a b) (+ a b))))
(def t8 (current-timestamp))

(print (string-append "  Pipeline time: " (string-append (number->string (- t8 t7)) "ms")))
(print (string-append "  Final result: " (number->string reduced)))
(print (string-append "  Items after filter: " (number->string (list-length filtered))))

;; Benchmark 5: Stress test with large dataset
(print "Benchmark 5: Stress test")
(def stress-data (build-list 5000))
(print "  Dataset size: 5000 items")

(def t9 (current-timestamp))
(def stress-result (pmap (lambda (x) (+ (* x x) (* x 2) 1)) stress-data))
(def t10 (current-timestamp))

(print (string-append "  pmap stress test time: " (string-append (number->string (- t10 t9)) "ms")))
(print (string-append "  Result size: " (number->string (list-length stress-result))))

;; Summary
(print "")
(print "=== Benchmark Summary ===")
(def total-time (+ (+ (+ (+ (- t2 t1) (- t4 t3)) (- t6 t5)) (- t8 t7)) (- t10 t9)))
(print (string-append "Total execution time: " (string-append (number->string total-time) "ms")))

(print "")
(print "Note: These benchmarks test pmap, pfilter, and preduce.")
(print "Performance will vary based on CPU cores and workload characteristics.")
(print "=== Benchmarks completed ===")
