; CLI File Processor: Reads a file and optionally writes to another
; Usage: lisp-vm cli-file-processor.bc <input-file> [output-file]

(defun get-first (lst)
  (car lst))

(defun get-second (lst)
  (car (cdr lst)))

(defun count-args (lst)
  (if (== lst '())
    0
    (+ 1 (count-args (cdr lst)))))

(defun process ()
  (let ((args (get-args)))
    (if (== (count-args args) 0)
      (print "Usage: cli-file-processor <input-file> [output-file]")
      (let ((input-file (get-first args)))
        (if (file-exists? input-file)
          (let ((content (read-file input-file)))
            (if (== (count-args args) 1)
              (print content)
              (write-file (get-second args) content)))
          (print (string-append "Error: File not found - " input-file)))))))

; Run the processor
(process)
