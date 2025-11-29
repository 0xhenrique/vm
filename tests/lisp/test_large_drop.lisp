;; Test iterative Drop implementation with large lists
;; This would cause stack overflow with the old recursive Drop implementation
;; EXPECT: success

;; Build a large list iteratively using loop/recur (no stack growth during build)
(defun build-list (n)
  (loop ((i 0) (acc '()))
    (if (>= i n)
        acc
        (recur (+ i 1) (cons i acc)))))

;; Test 1: Create and drop a large list (500k items)
;; List is dropped when let scope ends
(print "Test 1: Building 500k item list...")
(let ((large-list (build-list 500000)))
  (print (string-append "Built list with "
           (string-append (number->string (list-length large-list)) " items"))))
(print "Large list dropped successfully")

;; Test 2: Create an even larger list (1M items)
(print "Test 2: Building 1M item list...")
(let ((very-large-list (build-list 1000000)))
  (print (string-append "Built list with "
           (string-append (number->string (list-length very-large-list)) " items"))))
(print "Very large list dropped successfully")

;; Test 3: Shared tail test - create lists that share structure
(print "Test 3: Testing shared tails...")
(let ((shared-tail (build-list 100000)))
  (do
    ;; Create lists that share the tail
    (let ((list-a (cons 'a shared-tail))
          (list-b (cons 'b shared-tail))
          (list-c (cons 'c shared-tail)))
      (do
        (print (string-append "list-a length: " (number->string (list-length list-a))))
        (print (string-append "list-b length: " (number->string (list-length list-b))))
        (print (string-append "list-c length: " (number->string (list-length list-c))))))
    ;; list-a, list-b, list-c are now dropped, but shared-tail is still in scope
    (print (string-append "After inner scope, shared tail length: "
             (number->string (list-length shared-tail))))))
;; Now shared-tail is also dropped
(print "All shared lists dropped successfully")

(print "success")
