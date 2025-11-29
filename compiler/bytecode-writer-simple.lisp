; ============================================================================
; Bytecode Serialization Library - Simple Version
; ============================================================================

; ============================================================================
; Byte List Helpers
; ============================================================================

(defun append-bytes (bytes1 bytes2)
  (append bytes1 bytes2))

; ============================================================================
; Little-endian Number Serialization
; ============================================================================

; Convert u32 to 4 bytes (little-endian)
(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

; Convert i64 to 8 bytes (little-endian)
; For now, only handles non-negative integers
(defun i64-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (% (/ n 16777216) 256)
          (cons (% (/ n 4294967296) 256)
            (cons (% (/ n 1099511627776) 256)
              (cons (% (/ n 281474976710656) 256)
                (cons (/ n 72057594037927936) '())))))))))

; ============================================================================
; List Length Helper
; ============================================================================

(defun list-length (lst)
  (if (== lst '())
    0
    (+ 1 (list-length (cdr lst)))))

; ============================================================================
; Bytecode File Header
; ============================================================================

(defun make-bytecode-header ()
  (cons 76 (cons 73 (cons 83 (cons 80 (cons 6 '()))))))  ; "LISP" + version 6

; ============================================================================
; Test
; ============================================================================

(print "Bytecode writer library loaded")
(print "Testing u32-to-bytes:")
(print (u32-to-bytes 0))          ; (0 0 0 0)
(print (u32-to-bytes 255))        ; (255 0 0 0)
(print (u32-to-bytes 256))        ; (0 1 0 0)
(print (u32-to-bytes 65536))      ; (0 0 1 0)

(print "Testing header:")
(print (make-bytecode-header))    ; (76 73 83 80 6)

(print "Testing i64-to-bytes:")
(print (i64-to-bytes 0))
(print (i64-to-bytes 42))
(print (i64-to-bytes 1000))
