; Demonstration: Simple source code processor using file I/O
; This shows we're on the path to self-hosting!

(print "=== V19 Self-Hosting Demo: Source Code Processor ===")
(print "")

; ============================================================================
; Simple Lisp source analyzer
; ============================================================================

(defun count-chars
  ((str) (string-length str)))

(defun search-defun-helper
  ((s pos)
    (if (>= pos (- (string-length s) 4))
      false
      (if (== (substring s pos (+ pos 5)) "defun")
        true
        (search-defun-helper s (+ pos 1))))))

(defun contains-defun?
  ((str) (search-defun-helper str 0)))

; Read a Lisp source file and analyze it
(defvar source-file "tests/test-v18-cons-patterns.lisp")
(defvar source-code (read-file source-file))

(print "Analyzing file: tests/test-v18-cons-patterns.lisp")
(print (string-append "File size: " (string-append (if (> (count-chars source-code) 0) "non-empty" "empty") " file")))
(print (string-append "Contains 'defun': " (if (contains-defun? source-code) "yes" "no")))
(print "")

; ============================================================================
; Write generated code
; ============================================================================

(defvar output-path "/tmp/generated-lisp.lisp")
(defvar line1 "; Generated code\n")
(defvar line2 "(print 42)\n")
(defvar generated-code (string-append line1 line2))

(print "Writing generated code to /tmp/generated-lisp.lisp")
(print (write-file output-path generated-code))
(print "")

; Read it back and display
(print "Generated code:")
(print (read-file output-path))
(print "")

(print "✓ Self-hosting capabilities demonstrated!")
(print "  - Can read Lisp source files")
(print "  - Can analyze/process source code")
(print "  - Can generate and write Lisp code")
(print "  - Next step: Implement bytecode generation in Lisp!")
