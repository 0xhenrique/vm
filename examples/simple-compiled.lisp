; Simplest possible: Generate bytecode for print message program
; Bypasses complex serialization - just builds bytes directly

(defun append-bytes (b1 b2)
  (if (== b1 '()) b2
    (cons (car b1) (append-bytes (cdr b1) b2))))

(defun u32 (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

(defun str-bytes (s)
  (chars-bytes (string->list s)))

(defun chars-bytes (cs)
  (if (== cs '())
    '()
    (cons (char-code (car cs))
          (chars-bytes (cdr cs)))))

(defun make-prog ()
  (let ((msg "Success from Lisp compiler!"))
  (let ((msg-bytes (str-bytes msg)))
  (let ((msg-len (list-len msg-bytes)))
    (append-bytes '(76 73 83 80 6)      ; Header
      (append-bytes (u32 0)              ; 0 functions
        (append-bytes (u32 3)            ; 3 instructions
          (append-bytes '(0 4)           ; Push String
            (append-bytes (u32 msg-len)
              (append-bytes msg-bytes
                '(11 12)))))))))))       ; Print Halt

(defun list-len (l)
  (if (== l '()) 0 (+ 1 (list-len (cdr l)))))

(write-binary-file "/tmp/lisp-compiled.bc" (make-prog))
(print "Wrote /tmp/lisp-compiled.bc - Run it to see compilation in action!")
