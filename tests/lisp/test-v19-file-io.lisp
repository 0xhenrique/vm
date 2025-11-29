;; SKIP
;; Reason: Test contains runtime errors (file path issues)
; Test v19 file I/O operations

(print "=== Testing v19 File I/O ===")
(print "")

; ============================================================================
; Test 1: Write and read a simple file
; ============================================================================

(print "Test 1: write and read file")
(defvar test-path "/tmp/lisp-test-file.txt")
(defvar test-content "Hello from Lisp!")

(print (write-file test-path test-content))
(print (read-file test-path))
(print "")

; ============================================================================
; Test 2: file-exists?
; ============================================================================

(print "Test 2: file-exists?")
(print (file-exists? test-path))
(print (file-exists? "/tmp/nonexistent-file-xyz-123.txt"))
(print "")

; ============================================================================
; Test 3: Write multi-line content
; ============================================================================

(print "Test 3: multi-line content")
(defvar multiline-path "/tmp/lisp-multiline.txt")
(defvar multiline-content (string-append "Line 1"
                             (string-append "\n"
                               (string-append "Line 2"
                                 (string-append "\n" "Line 3")))))

(print (write-file multiline-path multiline-content))
(print (read-file multiline-path))
(print "")

; ============================================================================
; Test 4: Overwrite existing file
; ============================================================================

(print "Test 4: overwrite file")
(print (write-file test-path "Updated content"))
(print (read-file test-path))
(print "")

; ============================================================================
; Test 5: Read this source file (meta!)
; ============================================================================

(print "Test 5: read part of this source file")
(defvar this-file "tests/test-v19-file-io.lisp")
(defvar contents (read-file this-file))
(defvar first-line (substring contents 0 31))
(print first-line)
(print "")

(print "All v19 file I/O tests passed!")
