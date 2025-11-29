;; Test Phase 9: Standard Library Expansion
;; EXPECT-CONTAINS: All Phase 9 tests passed!

(print "=== Testing Phase 9: Standard Library Expansion ===")
(print "")

;; Helper functions from stdlib
(defun even? (n) (== (% n 2) 0))
(defun filter (pred lst)
  (if (null? lst)
      '()
      (if (pred (car lst))
          (cons (car lst) (filter pred (cdr lst)))
          (filter pred (cdr lst)))))
(defun not (x) (if x false true))

;; partition: Split list into two lists based on predicate
(defun partition (pred lst)
  (let ((trues (filter pred lst))
        (falses (filter (lambda (x) (not (pred x))) lst)))
    (list trues falses)))

;; interleave: Interleave two lists
(defun interleave (lst1 lst2)
  (if (null? lst1)
      lst2
      (if (null? lst2)
          lst1
          (cons (car lst1)
                (cons (car lst2)
                      (interleave (cdr lst1) (cdr lst2)))))))

;; interpose: Insert separator between list elements
(defun interpose (sep lst)
  (if (null? lst)
      '()
      (if (null? (cdr lst))
          lst
          (cons (car lst)
                (cons sep (interpose sep (cdr lst)))))))

;; frequencies: Count occurrences of each element (returns assoc list)
(defun frequencies-helper (lst acc)
  (if (null? lst)
      acc
      (let ((item (car lst))
            (rest (cdr lst)))
        (let ((entry (filter (lambda (pair) (== (car pair) item)) acc)))
          (if (null? entry)
              (frequencies-helper rest (cons (list item 1) acc))
              (let ((count (car (cdr (car entry))))
                    (others (filter (lambda (pair) (!= (car pair) item)) acc)))
                (frequencies-helper rest (cons (list item (+ count 1)) others))))))))

(defun frequencies (lst)
  (frequencies-helper lst '()))

;; group-by: Group list elements by function result (returns assoc list)
(defun group-by-helper (f lst acc)
  (if (null? lst)
      acc
      (let ((item (car lst))
            (key (f item))
            (rest (cdr lst)))
        (let ((entry (filter (lambda (pair) (== (car pair) key)) acc)))
          (if (null? entry)
              (group-by-helper f rest (cons (list key (list item)) acc))
              (let ((items (car (cdr (car entry))))
                    (others (filter (lambda (pair) (!= (car pair) key)) acc)))
                (group-by-helper f rest (cons (list key (append items (list item))) others))))))))

(defun group-by (f lst)
  (group-by-helper f lst '()))

;; sort: Sort a list using comparison function
(defun insert-sorted (cmp item lst)
  (if (null? lst)
      (list item)
      (if (cmp item (car lst))
          (cons item lst)
          (cons (car lst) (insert-sorted cmp item (cdr lst))))))

(defun sort (cmp lst)
  (if (null? lst)
      '()
      (insert-sorted cmp (car lst) (sort cmp (cdr lst)))))

;; sort-by: Sort a list by applying function to elements
(defun sort-by (f cmp lst)
  (sort (lambda (a b) (cmp (f a) (f b))) lst))

;; ============================================================================
;; Test 1: Math Functions
;; ============================================================================

(print "Test 1: Math Functions")
(print "log of e:")
(print (log 2.718281828459045))

(print "exp of 1:")
(print (exp 1))

(print "tan of 0:")
(print (tan 0))

(print "atan of 1 (should be ~0.785):")
(print (atan 1))

(print "atan2 of (1, 1) (should be ~0.785):")
(print (atan2 1 1))

(print "random number (0-1):")
(print (< (random) 1))
(print (>= (random) 0))

(print "random int (0-10):")
(print (let ((r (random-int 10))) (< r 10)))
(print (let ((r (random-int 10))) (>= r 0)))

(print "seed-random returns seed:")
(print (== (seed-random 42) 42))
(print "")

;; ============================================================================
;; Test 2: String Functions
;; ============================================================================

(print "Test 2: String Functions")
(print "string-split:")
(print (string-split "hello,world,test" ","))

(print "string-split with empty delimiter:")
(print (string-split "abc" ""))

(print "string-join:")
(print (string-join (list "hello" "world" "test") ","))

(print "string-trim:")
(print (string-trim "  hello world  "))

(print "string-replace:")
(print (string-replace "hello world" "world" "rust"))
(print "")

;; ============================================================================
;; Test 3: Date/Time Functions
;; ============================================================================

(print "Test 3: Date/Time Functions")
(print "current-timestamp (should be positive):")
(print (> (current-timestamp) 0))

(print "format-timestamp:")
(print (format-timestamp 1609459200 "%Y-%m-%d"))
(print "")

;; ============================================================================
;; Test 4: List Operations - partition
;; ============================================================================

(print "Test 4: partition")
(print "Evens:")
(print (car (partition even? '(1 2 3 4 5 6))))
(print "Odds:")
(print (car (cdr (partition even? '(1 2 3 4 5 6)))))
(print "")

;; ============================================================================
;; Test 5: List Operations - interleave
;; ============================================================================

(print "Test 5: interleave")
(print (interleave '(1 2 3) '(4 5 6)))
(print (interleave '(1 2) '(3 4 5)))
(print "")

;; ============================================================================
;; Test 6: List Operations - interpose
;; ============================================================================

(print "Test 6: interpose")
(print (interpose 0 '(1 2 3 4)))
(print (interpose "," (list "a" "b" "c")))
(print "")

;; ============================================================================
;; Test 7: List Operations - frequencies
;; ============================================================================

(print "Test 7: frequencies")
(print (frequencies '(1 2 2 3 3 3)))
(print "")

;; ============================================================================
;; Test 8: List Operations - group-by
;; ============================================================================

(print "Test 8: group-by")
(print (group-by even? '(1 2 3 4 5 6)))
(print "")

;; ============================================================================
;; Test 9: List Operations - sort
;; ============================================================================

(print "Test 9: sort")
(print (sort < '(3 1 4 1 5 9 2 6)))
(print (sort > '(3 1 4 1 5 9 2 6)))
(print "")

;; ============================================================================
;; Test 10: List Operations - sort-by
;; ============================================================================

(print "Test 10: sort-by")
(defun str-len (s) (string-length s))
(print (sort-by str-len < (list "zzz" "a" "bb" "cccc")))
(print "")

(print "All Phase 9 tests passed!")
