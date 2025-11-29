; Test v19 string operations

(print "=== Testing v19 String Operations ===")
(print "")

; ============================================================================
; Test 1: string-length
; ============================================================================

(print "Test 1: string-length")
(print (string-length ""))
(print (string-length "hello"))
(print (string-length "hello world"))
(print "")

; ============================================================================
; Test 2: substring
; ============================================================================

(print "Test 2: substring")
(print (substring "hello world" 0 5))
(print (substring "hello world" 6 11))
(print (substring "hello world" 0 11))
(print (substring "abcdef" 2 4))
(print "")

; ============================================================================
; Test 3: string-append
; ============================================================================

(print "Test 3: string-append")
(print (string-append "hello" " world"))
(print (string-append "foo" "bar"))
(print (string-append "" "test"))
(print (string-append "test" ""))
(print "")

; ============================================================================
; Test 4: string->list
; ============================================================================

(print "Test 4: string->list")
(print (string->list ""))
(print (string->list "abc"))
(print (string->list "hello"))
(print "")

; ============================================================================
; Test 5: list->string
; ============================================================================

(defun make-abc () (string->list "abc"))
(defun make-hello () (string->list "hello"))

(print "Test 5: list->string")
(print (list->string '()))
(print (list->string (make-abc)))
(print (list->string (make-hello)))
(print "")

; ============================================================================
; Test 6: Round-trip string->list->string
; ============================================================================

(defun str-roundtrip
  ((s) (list->string (string->list s))))

(print "Test 6: round-trip string->list->string")
(print (str-roundtrip ""))
(print (str-roundtrip "test"))
(print (str-roundtrip "hello world"))
(print "")

; ============================================================================
; Test 7: String utilities with pattern matching
; ============================================================================

(defun string-first
  ((s)
    (if (== (string-length s) 0)
      ""
      (substring s 0 1))))

(defun string-rest
  ((s)
    (if (== (string-length s) 0)
      ""
      (substring s 1 (string-length s)))))

(print "Test 7: string utilities")
(print (string-first "hello"))
(print (string-first ""))
(print (string-rest "hello"))
(print (string-rest "h"))
(print (string-rest ""))
(print "")

; ============================================================================
; Test 8: Build string from list
; ============================================================================

(defun build-greeting
  ((name)
    (string-append "Hello, "
      (string-append name "!"))))

(print "Test 8: build greeting")
(print (build-greeting "World"))
(print (build-greeting "Alice"))
(print "")

(print "All v19 string tests passed!")
