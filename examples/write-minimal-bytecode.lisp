; Create a minimal valid bytecode file that prints "Hello!"
; This demonstrates the complete bytecode serialization capability

; List append function
(defun append (lst1 lst2)
  (if (== lst1 '())
    lst2
    (cons (car lst1) (append (cdr lst1) lst2))))

; Include byte serialization functions
(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

(defun i64-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (% (/ n 16777216) 256)
          (cons (% (/ n 4294967296) 256)
            (cons (% (/ n 1099511627776) 256)
              (cons (% (/ n 281474976710656) 256)
                (cons (/ n 72057594037927936) '())))))))))

(defun append-bytes (bytes1 bytes2)
  (append bytes1 bytes2))

; Create bytecode header: "LISP" + version 6
(defun make-header ()
  (cons 76 (cons 73 (cons 83 (cons 80 (cons 6 '()))))))

; Create a minimal program:
;   0 functions
;   Main: Push "Hello!" then Print then Halt
(defun make-minimal-program ()
  (let ((header (make-header)))
  (let ((func-count (u32-to-bytes 0)))  ; 0 functions
  (let ((main-count (u32-to-bytes 3)))  ; 3 instructions in main
  (let ((push-instr (cons 0 (cons 4 (serialize-hello)))))  ; Push string "Hello!"
  (let ((print-instr '(11)))             ; Print opcode
  (let ((halt-instr '(12)))              ; Halt opcode
    (append-bytes header
      (append-bytes func-count
        (append-bytes main-count
          (append-bytes push-instr
            (append-bytes print-instr halt-instr))))))))))))

; Serialize the string "Hello!" for bytecode
; String format: u32 length + ASCII bytes
(defun serialize-hello ()
  (let ((length (u32-to-bytes 6)))  ; "Hello!" is 6 chars
  (let ((bytes '(72 101 108 108 111 33)))  ; "Hello!" in ASCII
    (append-bytes length bytes))))

; Main: Generate and write the bytecode file
(defun main ()
  (let ((bytecode (make-minimal-program)))
    (if (write-binary-file "/tmp/hello.bc" bytecode)
      (print "Successfully wrote /tmp/hello.bc")
      (print "Failed to write bytecode"))))

(main)
