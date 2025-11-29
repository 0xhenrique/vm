;; ============================================================
;; Standard Library for Lisp Bytecode VM
;; ============================================================

;; ------------------------------------------------------------
;; List Utilities
;; ------------------------------------------------------------

;; map: Apply function to each element of a list
(defun map (f lst)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;; filter: Keep only elements that satisfy predicate
(defun filter (pred lst)
  (if (null? lst)
      '()
      (if (pred (car lst))
          (cons (car lst) (filter pred (cdr lst)))
          (filter pred (cdr lst)))))

;; reduce/fold: Reduce list with binary function and initial value
(defun reduce (f init lst)
  (if (null? lst)
      init
      (reduce f (f init (car lst)) (cdr lst))))

;; length: Get length of a list
(defun length (lst)
  (if (null? lst)
      0
      (+ 1 (length (cdr lst)))))

;; reverse-helper: Helper for reverse
(defun reverse-helper (lst acc)
  (if (null? lst)
      acc
      (reverse-helper (cdr lst) (cons (car lst) acc))))

;; reverse: Reverse a list
(defun reverse (lst)
  (reverse-helper lst '()))

;; take: Take first n elements of a list
(defun take (n lst)
  (if (<= n 0)
      '()
      (if (null? lst)
          '()
          (cons (car lst) (take (- n 1) (cdr lst))))))

;; drop: Drop first n elements of a list
(defun drop (n lst)
  (if (<= n 0)
      lst
      (if (null? lst)
          '()
          (drop (- n 1) (cdr lst)))))

;; nth: Get nth element (0-indexed, alias for list-ref)
(defun nth (lst n)
  (list-ref lst n))

;; range: Create a list from start to end (exclusive)
(defun range (start end)
  (if (>= start end)
      '()
      (cons start (range (+ start 1) end))))

;; zip: Combine two lists into pairs
;; Note: Uses builtin 'list' function
(defun zip (lst1 lst2)
  (if (null? lst1)
      '()
      (if (null? lst2)
          '()
          (cons (list (car lst1) (car lst2))
                (zip (cdr lst1) (cdr lst2))))))

;; all?: Check if all elements satisfy predicate
(defun all? (pred lst)
  (if (null? lst)
      true
      (if (pred (car lst))
          (all? pred (cdr lst))
          false)))

;; any?: Check if any element satisfies predicate
(defun any? (pred lst)
  (if (null? lst)
      false
      (if (pred (car lst))
          true
          (any? pred (cdr lst)))))

;; ------------------------------------------------------------
;; Math Utilities
;; ------------------------------------------------------------

;; abs: Absolute value
(defun abs (x)
  (if (< x 0)
      (neg x)
      x))

;; min: Minimum of two numbers
(defun min (a b)
  (if (< a b) a b))

;; max: Maximum of two numbers
(defun max (a b)
  (if (> a b) a b))

;; even?: Check if number is even
(defun even? (n)
  (== (% n 2) 0))

;; odd?: Check if number is odd
(defun odd? (n)
  (!= (% n 2) 0))

;; sum: Sum all numbers in a list
(defun sum (lst)
  (reduce + 0 lst))

;; product: Product of all numbers in a list
(defun product (lst)
  (reduce * 1 lst))

;; ------------------------------------------------------------
;; Higher-Order Functions
;; ------------------------------------------------------------

;; compose: Compose two functions (f . g)(x) = f(g(x))
(defun compose (f g)
  (lambda (x) (f (g x))))

;; identity: Identity function
(defun identity (x) x)

;; constantly: Return a function that always returns the same value
(defun constantly (x)
  (lambda (_) x))

;; ------------------------------------------------------------
;; Predicates
;; ------------------------------------------------------------

;; null?: Check if list is empty
(defun null? (lst)
  (== (list-length lst) 0))

;; empty?: Alias for null?
(defun empty? (lst)
  (null? lst))

;; not: Logical NOT
(defun not (x)
  (if x false true))

;; partition: Split list into two lists based on predicate
;; Returns (truthy-items falsy-items)
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
;; Helper function for frequencies
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
;; Uses a simple insertion sort
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

;; ------------------------------------------------------------
;; String Utilities
;; ------------------------------------------------------------

;; string-empty?: Check if string is empty
(defun string-empty? (s)
  (== (string-length s) 0))

;; string-first: Get first character of string
(defun string-first (s)
  (if (string-empty? s)
      ""
      (substring s 0 1)))

;; string-rest: Get all but first character
(defun string-rest (s)
  (if (string-empty? s)
      ""
      (substring s 1 (string-length s))))

;; ------------------------------------------------------------
;; Error Handling - Errors as Values (Rust/Go style)
;; ------------------------------------------------------------

;; Result type constructors
;; A Result is either (ok value) or (err message)

;; ok: Create a successful result
(defun ok (value)
  (list 'ok value))

;; err: Create an error result
(defun err (message)
  (list 'err message))

;; Result predicates

;; result?: Check if a value is a result (either ok or err)
(defun result? (r)
  (if (list? r)
      (if (>= (list-length r) 1)
          (let ((tag (car r)))
            (if (== tag 'ok)
                true
                (== tag 'err)))
          false)
      false))

;; ok?: Check if a result is ok
(defun ok? (r)
  (if (list? r)
      (if (>= (list-length r) 1)
          (== (car r) 'ok)
          false)
      false))

;; err?: Check if a result is err
(defun err? (r)
  (if (list? r)
      (if (>= (list-length r) 1)
          (== (car r) 'err)
          false)
      false))

;; Result value extraction

;; unwrap: Extract value from ok result, panic on err
(defun unwrap (r)
  (if (ok? r)
      (car (cdr r))
      (print (if (err? r)
                 (string-append "PANIC: Cannot unwrap error result: " (car (cdr r)))
                 "PANIC: Cannot unwrap error result: not a result type"))))

;; unwrap-or: Extract value from ok, or return default on err
(defun unwrap-or (r default)
  (if (ok? r)
      (car (cdr r))
      default))

;; unwrap-err: Extract error message from err result, panic on ok
(defun unwrap-err (r)
  (if (err? r)
      (car (cdr r))
      (print "PANIC: Cannot unwrap-err from ok result")))

;; expect: Extract value from ok, panic with custom message on err
(defun expect (r message)
  (if (ok? r)
      (car (cdr r))
      (print (if (err? r)
                 (string-append "PANIC: " (string-append message (string-append ": " (car (cdr r)))))
                 (string-append "PANIC: " (string-append message ": not a result type"))))))

;; expect-err: Extract error from err, panic with custom message on ok
(defun expect-err (r message)
  (if (err? r)
      (car (cdr r))
      (print (string-append "PANIC: " message))))

;; Result transformations

;; map-ok: Apply function to ok value, pass through err unchanged
(defun map-ok (f r)
  (if (ok? r)
      (ok (f (car (cdr r))))
      r))

;; map-err: Apply function to err message, pass through ok unchanged
(defun map-err (f r)
  (if (err? r)
      (err (f (car (cdr r))))
      r))

;; and-then: Chain operations that return results (flatMap/bind)
;; If r is ok, apply f to its value (f must return a result)
;; If r is err, pass through the error
(defun and-then (r f)
  (if (ok? r)
      (f (car (cdr r)))
      r))

;; or-else: Recover from errors
;; If r is err, apply f to its error message (f must return a result)
;; If r is ok, pass through the value
(defun or-else (r f)
  (if (err? r)
      (f (car (cdr r)))
      r))

;; Helper: is-ok (alias for ok?)
(defun is-ok (r) (ok? r))

;; Helper: is-err (alias for err?)
(defun is-err (r) (err? r))
;; Benchmark: Memory Pressure
;; Tests memory allocation patterns and potential bottlenecks

(print "=== Benchmark: Memory Pressure ===")
(print "")

;; ============================================================================
;; Benchmark 1: Large List Allocation
;; ============================================================================
(print "Benchmark 1: Large List Allocation")

(defun make-list (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons i acc)))))

(print "Allocating list of 50,000 elements...")
(def large-list (make-list 50000))
(print (string-append "List created. Length: " (number->string (length large-list))))
(print "")

;; ============================================================================
;; Benchmark 2: Many Small Lists
;; ============================================================================
(print "Benchmark 2: Many Small Lists")

(defun create-many-lists (count size)
  (loop ((i 0) (acc '()))
    (if (>= i count)
        acc
        (recur (+ i 1) (cons (make-list size) acc)))))

(print "Creating 1000 lists of 100 elements each...")
(def many-lists (create-many-lists 1000 100))
(print (string-append (string-append "Created " (number->string (length many-lists))) " lists"))
(print "")

;; ============================================================================
;; Benchmark 3: String Allocation
;; ============================================================================
(print "Benchmark 3: String Operations")

(defun repeat-string (s n)
  (loop ((i 0) (acc ""))
    (if (>= i n)
        acc
        (recur (+ i 1) (string-append acc s)))))

(print "Building string by repeated append (100 iterations)...")
(def long-string (repeat-string "Hello" 100))
(print (string-append "String length: " (number->string (string-length long-string))))
(print "")

;; ============================================================================
;; Benchmark 4: List of Strings
;; ============================================================================
(print "Benchmark 4: List of Strings")

(defun make-string-list (count)
  (loop ((i 0) (acc '()))
    (if (>= i count)
        acc
        (recur (+ i 1)
               (cons (string-append "item-" (number->string i)) acc)))))

(print "Creating list of 5000 strings...")
(def string-list (make-string-list 5000))
(print (string-append (string-append "Created " (number->string (length string-list))) " strings"))
(print (string-append "First: " (car string-list)))
(print "")

;; ============================================================================
;; Benchmark 5: Nested Data Structures
;; ============================================================================
(print "Benchmark 5: Nested Data Structures")

(defun make-tree (depth)
  (if (<= depth 0)
      'leaf
      (list 'node (make-tree (- depth 1)) (make-tree (- depth 1)))))

(print "Creating binary tree of depth 12 (4095 nodes)...")
(def tree (make-tree 12))

(defun count-nodes (t)
  (if (== t 'leaf)
      1
      (+ 1 (+ (count-nodes (car (cdr t)))
              (count-nodes (car (cdr (cdr t))))))))

(print (string-append "Node count: " (number->string (count-nodes tree))))
(print "")

;; ============================================================================
;; Benchmark 6: Data Transformation Pipeline
;; ============================================================================
(print "Benchmark 6: Data Transformation Pipeline")

(def pipeline-data (make-list 10000))

(print "Running: map -> filter -> map -> reduce")

(def pipeline-result
  (reduce + 0
    (map (lambda (x) (* x 2))
      (filter even?
        (map (lambda (x) (+ x 1)) pipeline-data)))))

(print (string-append "Pipeline result: " (number->string pipeline-result)))
(print "")

;; ============================================================================
;; Benchmark 7: Hash Map Operations
;; ============================================================================
(print "Benchmark 7: Hash Map Operations")

(defun make-hash-entries (count)
  (loop ((i 0) (acc '()))
    (if (>= i count)
        acc
        (recur (+ i 1)
               (cons (list (string-append "key-" (number->string i)) i) acc)))))

(print "Creating 1000 key-value pairs...")
(def entries (make-hash-entries 1000))
(print (string-append "Entries created: " (number->string (length entries))))
(print "")

;; ============================================================================
;; Benchmark 8: Intermediate Allocations
;; ============================================================================
(print "Benchmark 8: Intermediate Allocations (Worst Case)")

;; This pattern creates many intermediate lists
(defun wasteful-sum (lst)
  (if (null? lst)
      0
      (+ (car lst)
         (wasteful-sum (cdr (append '(0) lst))))))  ;; Creates new list each time!

;; Only use small input - this is O(n^2) in allocations
(print "Running wasteful pattern on 100 elements...")
(def wasteful-input (make-list 100))
;; Skip this - too slow: (def wasteful-result (wasteful-sum wasteful-input))

;; Better pattern
(defun efficient-sum (lst)
  (loop ((remaining lst) (acc 0))
    (if (null? remaining)
        acc
        (recur (cdr remaining) (+ acc (car remaining))))))

(print (string-append "Efficient sum: " (number->string (efficient-sum wasteful-input))))
(print "")

;; ============================================================================
;; Benchmark 9: Repeated Access Pattern
;; ============================================================================
(print "Benchmark 9: Repeated Access Pattern")

(def access-list (make-list 1000))

(defun access-many-times (lst count)
  (loop ((i 0) (acc 0))
    (if (>= i count)
        acc
        (recur (+ i 1) (+ acc (car lst))))))

(print "Accessing first element 10000 times...")
(def access-result (access-many-times access-list 10000))
(print (string-append "Sum of accesses: " (number->string access-result)))
(print "")

;; ============================================================================
;; Benchmark 10: GC Pressure Simulation
;; ============================================================================
(print "Benchmark 10: Allocation Churn")

;; Create and discard many temporary lists
(defun churn (iterations list-size)
  (loop ((i 0))
    (if (>= i iterations)
        'done
        (let ((temp (make-list list-size)))  ;; Allocate
          (recur (+ i 1))))))               ;; Discard

(print "Creating and discarding 100 lists of 1000 elements...")
(churn 100 1000)
(print "Churn complete (Rust should have freed all temporaries)")
(print "")

(print "=== Memory Pressure Benchmark Complete ===")
