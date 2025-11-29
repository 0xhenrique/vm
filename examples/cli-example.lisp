; CLI Example: Demonstrates command-line argument handling
; Usage: lisp-vm cli-example.bc <input-file> [output-file]

(defun main ()
  (let ((args (get-args)))
    (if (== args '())
      (print-usage)
      (process-files args))))

(defun print-usage ()
  (print "Usage: cli-example <input-file> [output-file]")
  (print "  <input-file>  - File to read")
  (print "  [output-file] - Optional output file (default: stdout)"))

(defun process-files (args)
  (let ((input-file (car args)))
    (if (file-exists? input-file)
      (let ((content (read-file input-file)))
        (print "File contents:")
        (print content)
        (if (> (list-length args) 1)
          (write-to-output (car (cdr args)) content)
          (print "No output file specified")))
      (print (string-append "Error: File not found: " input-file)))))

(defun write-to-output (output-file content)
  (write-file output-file content)
  (print (string-append "Written to: " output-file)))

(defun list-length (lst)
  (if (== lst '())
    0
    (+ 1 (list-length (cdr lst)))))

; Run the main function
(main)
