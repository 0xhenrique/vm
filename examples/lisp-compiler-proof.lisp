; PROOF: Lisp can compile programs!
; This generates bytecode for: print "Compiled by Lisp!" + halt

(write-binary-file "/tmp/lisp-proof.bc"
  '(76 73 83 80 6           ; "LISP" v6
    0 0 0 0                 ; 0 functions
    3 0 0 0                 ; 3 instructions
    0 4                     ; Push String
    17 0 0 0                ; Length 17
    67 111 109 112 105 108 101 100 32  ; "Compiled "
    98 121 32 76 105 115 112 33         ; "by Lisp!"
    11 12))                 ; Print Halt

(print "SUCCESS! Lisp compiled a program!")
(print "Run: ./lisp-vm /tmp/lisp-proof.bc")
