; Test write-binary-file
; Should write "LISP" magic number to file

(defun test-write-binary ()
  (let ((bytes '(76 73 83 80 6)))  ; "LISP" + version 6
    (if (write-binary-file "/tmp/test.bin" bytes)
      (print "Wrote binary file to /tmp/test.bin")
      (print "Failed to write file"))))

(test-write-binary)
