; Simple test of bytecode serialization components

(defun append (lst1 lst2)
  (if (== lst1 '())
    lst2
    (cons (car lst1) (append (cdr lst1) lst2))))

(defun list-length (lst)
  (if (== lst '())
    0
    (+ 1 (list-length (cdr lst)))))

; Convert u32 to 4 bytes
(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

; Convert string to bytes using char-code
(defun string-to-bytes (s)
  (string-chars-to-bytes (string->list s)))

(defun string-chars-to-bytes (chars)
  (if (== chars '())
    '()
    (cons (char-code (car chars))
          (string-chars-to-bytes (cdr chars)))))

; Serialize string: length + bytes
(defun serialize-string (s)
  (let ((bytes (string-to-bytes s)))
    (append (u32-to-bytes (list-length bytes)) bytes)))

; Create header
(defun make-header ()
  '(76 73 83 80 6))

; Serialize simple "Hello!" program
(defun make-hello-program ()
  (append (make-header)
    (append (u32-to-bytes 0)  ; 0 functions
      (append (u32-to-bytes 3)  ; 3 instructions
        (append (make-push-hello)
          (append '(11)  ; Print
                  '(12)))))))  ; Halt

; Make Push "Hello!" instruction
(defun make-push-hello ()
  (append '(0)  ; Push opcode
    (append '(4)  ; String value tag
            (serialize-string "Hello!"))))

; Main: Generate and write bytecode
(defun main ()
  (let ((bytecode (make-hello-program)))
    (if (write-binary-file "/tmp/hello-gen.bc" bytecode)
      (print "SUCCESS! Generated /tmp/hello-gen.bc")
      (print "Failed to write bytecode"))))

(main)
