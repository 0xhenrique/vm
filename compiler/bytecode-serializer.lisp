; ============================================================================
; Complete Bytecode Serialization Library
; ============================================================================
; Handles serialization of all bytecode components to binary format

; ============================================================================
; Helper Functions
; ============================================================================

(defun append (lst1 lst2)
  (if (== lst1 '())
    lst2
    (cons (car lst1) (append (cdr lst1) lst2))))

(defun list-length (lst)
  (if (== lst '())
    0
    (+ 1 (list-length (cdr lst)))))

; ============================================================================
; Number Serialization (Little-Endian)
; ============================================================================

; Convert u32 to 4 bytes (little-endian)
(defun u32-to-bytes (n)
  (cons (% n 256)
    (cons (% (/ n 256) 256)
      (cons (% (/ n 65536) 256)
        (cons (/ n 16777216) '())))))

; Convert i64 to 8 bytes (little-endian)
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
; String Serialization
; ============================================================================

; Convert string to list of byte values
(defun string-to-bytes (s)
  (string-chars-to-bytes (string->list s)))

(defun string-chars-to-bytes (chars)
  (if (== chars '())
    '()
    (cons (char-code (car chars))
          (string-chars-to-bytes (cdr chars)))))

; Serialize string: u32 length + UTF-8 bytes
(defun serialize-string (s)
  (let ((bytes (string-to-bytes s)))
    (append (u32-to-bytes (list-length bytes)) bytes)))

; ============================================================================
; Bytecode File Header
; ============================================================================

(defun make-header ()
  '(76 73 83 80 6))  ; "LISP" + version 6

; ============================================================================
; Value Serialization
; ============================================================================
; Value tags:
;   0 = Integer (8 bytes i64-le)
;   1 = Boolean (1 byte: 0 or 1)
;   2 = List (u32 count + values)
;   3 = Symbol (string)
;   4 = String (string)

(defun serialize-value (val)
  (if (int? val)
    (cons 0 (i64-to-bytes val))
  (if (bool? val)
    (cons 1 (cons (if val 1 0) '()))
  (if (list? val)
    (cons 2 (append (u32-to-bytes (list-length val))
                    (serialize-value-list val)))
  (if (symbol? val)
    (cons 3 (serialize-string (symbol-to-string val)))
  (if (string? val)
    (cons 4 (serialize-string val))
    '()))))))

(defun serialize-value-list (vals)
  (if (== vals '())
    '()
    (append (serialize-value (car vals))
            (serialize-value-list (cdr vals)))))

; Type predicates
(defun int? (v)
  (if (list? v) false
  (if (bool? v) false
  (if (symbol? v) false
  (if (string? v) false
    true)))))

(defun bool? (v)
  (or (== v true) (== v false)))

; ============================================================================
; Instruction Serialization
; ============================================================================
; Opcodes: 0-50

(defun serialize-instruction (instr)
  (let ((op (car instr)))
    (if (== op 'push)
      (cons 0 (serialize-value (car (cdr instr))))
    (if (== op 'add) '(1)
    (if (== op 'sub) '(2)
    (if (== op 'mul) '(3)
    (if (== op 'div) '(4)
    (if (== op 'leq) '(5)
    (if (== op 'jmp-if-false)
      (cons 6 (u32-to-bytes (car (cdr instr))))
    (if (== op 'jmp)
      (cons 7 (u32-to-bytes (car (cdr instr))))
    (if (== op 'call)
      (append '(8) (append (serialize-string (car (cdr instr)))
                           (u32-to-bytes (car (cdr (cdr instr))))))
    (if (== op 'ret) '(9)
    (if (== op 'load-arg)
      (cons 10 (u32-to-bytes (car (cdr instr))))
    (if (== op 'print) '(11)
    (if (== op 'halt) '(12)
    (if (== op 'mod) '(13)
    (if (== op 'neg) '(14)
    (if (== op 'lt) '(15)
    (if (== op 'gt) '(16)
    (if (== op 'gte) '(17)
    (if (== op 'eq) '(18)
    (if (== op 'neq) '(19)
    (if (== op 'cons) '(20)
    (if (== op 'car) '(21)
    (if (== op 'cdr) '(22)
    (if (== op 'is-list) '(23)
    (if (== op 'is-string) '(24)
    (if (== op 'is-symbol) '(25)
    (if (== op 'symbol-to-string) '(26)
    (if (== op 'string-to-symbol) '(27)
    (if (== op 'get-local)
      (cons 28 (u32-to-bytes (car (cdr instr))))
    (if (== op 'string-length) '(40)
    (if (== op 'substring) '(41)
    (if (== op 'string-append) '(42)
    (if (== op 'string->list) '(43)
    (if (== op 'list->string) '(44)
    (if (== op 'char-code) '(50)
    (if (== op 'read-file) '(45)
    (if (== op 'write-file) '(46)
    (if (== op 'file-exists?) '(47)
    (if (== op 'get-args) '(48)
    (if (== op 'write-binary-file) '(49)
      '(255)  ; Unknown instruction
    )))))))))))))))))))))))))))))))))))))))))

(defun serialize-instruction-list (instrs)
  (if (== instrs '())
    '()
    (append (serialize-instruction (car instrs))
            (serialize-instruction-list (cdr instrs)))))

; ============================================================================
; Bytecode Serialization
; ============================================================================

(defun serialize-bytecode (instrs)
  (append (u32-to-bytes (list-length instrs))
          (serialize-instruction-list instrs)))

(defun serialize-function (name instrs)
  (append (serialize-string name)
          (serialize-bytecode instrs)))

; ============================================================================
; Complete Bytecode File
; ============================================================================

(defun serialize-bytecode-file (functions main-instrs)
  (append (make-header)
    (append (u32-to-bytes (list-length functions))
      (append (serialize-functions functions)
              (serialize-bytecode main-instrs)))))

(defun serialize-functions (functions)
  (if (== functions '())
    '()
    (append (serialize-function (car (car functions)) (car (cdr (car functions))))
            (serialize-functions (cdr functions)))))

; ============================================================================
; Tests
; ============================================================================

(print "Bytecode serializer loaded!")
(print "")
(print "Test serialize-string:")
(print (serialize-string "hi"))  ; Should be: (2 0 0 0 104 105)

(print "")
(print "Test serialize-value (integer):")
(print (serialize-value 42))  ; Should be: (0 42 0 0 0 0 0 0 0)

(print "")
(print "Test serialize-instruction (halt):")
(print (serialize-instruction '(halt)))  ; Should be: (12)

(print "")
(print "Test serialize-instruction (push 42):")
(print (serialize-instruction '(push 42)))  ; Should be: (0 0 42 0 0 0 0 0 0 0)
