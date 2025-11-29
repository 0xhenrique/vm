; Simpler bytecode writer to test step by step

(defun append (lst1 lst2)
  (if (== lst1 '())
    lst2
    (cons (car lst1) (append (cdr lst1) lst2))))

(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

; Test step 1: Create header
(defun test-header ()
  (cons 76 (cons 73 (cons 83 (cons 80 (cons 6 '()))))))

; Test step 2: Append header + function count
(defun test-header-and-count ()
  (append (test-header) (u32-to-bytes 0)))

; Test step 3: Complete minimal bytecode
(defun test-complete ()
  (let ((header (test-header)))
  (let ((with-funcs (append header (u32-to-bytes 0))))
  (let ((with-main-count (append with-funcs (u32-to-bytes 1))))
  (let ((with-halt (append with-main-count '(12))))
    with-halt)))))

; Main
(defun main ()
  (let ((bytecode (test-complete)))
    (if (write-binary-file "/tmp/simple.bc" bytecode)
      (print "Wrote /tmp/simple.bc")
      (print "Failed"))))

(main)
