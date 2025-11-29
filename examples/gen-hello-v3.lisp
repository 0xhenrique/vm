; Generate "Hello!" bytecode using proven approach

(defun append (lst1 lst2)
  (if (== lst1 '())
    lst2
    (cons (car lst1) (append (cdr lst1) lst2))))

(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

; Manually build "Hello!" bytecode
(defun make-hello-bc ()
  (let ((header '(76 73 83 80 6)))
  (let ((funcs (u32-to-bytes 0)))
  (let ((count (u32-to-bytes 3)))
  (let ((p1 '(0 4)))  ; Push String tag
  (let ((len (u32-to-bytes 6)))  ; "Hello!" length
  (let ((msg '(72 101 108 108 111 33)))  ; "Hello!" ASCII
  (let ((print-halt '(11 12)))  ; Print, Halt
    (append header
      (append funcs
        (append count
          (append p1
            (append len
              (append msg print-halt))))))))))))))

(defun main ()
  (write-binary-file "/tmp/hello3.bc" (make-hello-bc)))

(main)
