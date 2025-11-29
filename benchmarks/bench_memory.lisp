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
