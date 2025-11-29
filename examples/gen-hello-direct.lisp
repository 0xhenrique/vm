; Direct bytecode generation - all bytes inline

(defun make-bytecode ()
  '(76 73 83 80 6           ; "LISP" + version 6
    0 0 0 0                 ; 0 functions
    3 0 0 0                 ; 3 instructions in main
    0                       ; Push opcode
    4                       ; String value tag
    6 0 0 0                 ; String length = 6
    72 101 108 108 111 33   ; "Hello!" ASCII
    11                      ; Print opcode
    12))                    ; Halt opcode

(write-binary-file "/tmp/hello-direct.bc" (make-bytecode))
(print "Wrote /tmp/hello-direct.bc")
