; ============================================================================
; Bytecode Serialization Library
; ============================================================================
; Functions to serialize Lisp bytecode to binary format compatible with VM
;
; Format:
;   Magic: "LISP" (bytes: 76 73 83 80)
;   Version: 6 (byte)
;   Function count: u32-le
;   For each function:
;     Name: u32-le length + UTF-8 bytes
;     Bytecode: u32-le count + instructions
;   Main bytecode: u32-le count + instructions

; ============================================================================
; Byte List Helpers
; ============================================================================

(defun append-byte (bytes byte)
  (append bytes (cons byte '())))

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
(defun i64-to-bytes (n)
  (if (< n 0)
    (i64-to-bytes-impl (+ n 18446744073709551616))  ; Two's complement
    (i64-to-bytes-impl n)))

(defun i64-to-bytes-impl (n)
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

; Get ASCII code of a character (simplified - assumes single-char strings)
(defun char-code (c)
  (if (== c "a") 97
  (if (== c "b") 98
  (if (== c "c") 99
  (if (== c "d") 100
  (if (== c "e") 101
  (if (== c "f") 102
  (if (== c "g") 103
  (if (== c "h") 104
  (if (== c "i") 105
  (if (== c "j") 106
  (if (== c "k") 107
  (if (== c "l") 108
  (if (== c "m") 109
  (if (== c "n") 110
  (if (== c "o") 111
  (if (== c "p") 112
  (if (== c "q") 113
  (if (== c "r") 114
  (if (== c "s") 115
  (if (== c "t") 116
  (if (== c "u") 117
  (if (== c "v") 118
  (if (== c "w") 119
  (if (== c "x") 120
  (if (== c "y") 121
  (if (== c "z") 122
  (if (== c "A") 65
  (if (== c "B") 66
  (if (== c "C") 67
  (if (== c "D") 68
  (if (== c "E") 69
  (if (== c "F") 70
  (if (== c "G") 71
  (if (== c "H") 72
  (if (== c "I") 73
  (if (== c "J") 74
  (if (== c "K") 75
  (if (== c "L") 76
  (if (== c "M") 77
  (if (== c "N") 78
  (if (== c "O") 79
  (if (== c "P") 80
  (if (== c "Q") 81
  (if (== c "R") 82
  (if (== c "S") 83
  (if (== c "T") 84
  (if (== c "U") 85
  (if (== c "V") 86
  (if (== c "W") 87
  (if (== c "X") 88
  (if (== c "Y") 89
  (if (== c "Z") 90
  (if (== c "0") 48
  (if (== c "1") 49
  (if (== c "2") 50
  (if (== c "3") 51
  (if (== c "4") 52
  (if (== c "5") 53
  (if (== c "6") 54
  (if (== c "7") 55
  (if (== c "8") 56
  (if (== c "9") 57
  (if (== c " ") 32
  (if (== c "-") 45
  (if (== c "_") 95
  (if (== c ".") 46
  (if (== c "?") 63
  (if (== c "!") 33
  (if (== c ">") 62
  (if (== c "<") 60
  (if (== c "=") 61
  (if (== c "+") 43
  (if (== c "*") 42
  (if (== c "/") 47
  (if (== c "(") 40
  (if (== c ")") 41
  (if (== c "[") 91
  (if (== c "]") 93
  (if (== c ":") 58
  (if (== c ",") 44
    0)))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))

; Convert string to list of bytes
(defun string-to-bytes (s)
  (string-to-bytes-impl (string->list s)))

(defun string-to-bytes-impl (chars)
  (if (== chars '())
    '()
    (cons (char-code (car chars))
          (string-to-bytes-impl (cdr chars)))))

; Serialize string: u32 length + bytes
(defun serialize-string (s)
  (let ((bytes (string-to-bytes s)))
    (append-bytes (u32-to-bytes (list-length bytes)) bytes)))

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
    (cons 2 (append-bytes (u32-to-bytes (list-length val))
                          (serialize-value-list val)))
  (if (symbol? val)
    (cons 3 (serialize-string (symbol->string val)))
  (if (string? val)
    (cons 4 (serialize-string val))
    '()))))))  ; Unknown type

(defun serialize-value-list (vals)
  (if (== vals '())
    '()
    (append-bytes (serialize-value (car vals))
                  (serialize-value-list (cdr vals)))))

; ============================================================================
; Helper Predicates
; ============================================================================

(defun int? (v)
  (if (list? v) false
  (if (bool? v) false
  (if (symbol? v) false
  (if (string? v) false
    true)))))

(defun bool? (v)
  (or (== v true) (== v false)))

(defun symbol->string (s)
  (if (symbol? s)
    (symbol-to-string s)
    ""))

; ============================================================================
; List Length Helper
; ============================================================================

(defun list-length (lst)
  (if (== lst '())
    0
    (+ 1 (list-length (cdr lst)))))

; ============================================================================
; Test
; ============================================================================

(print "Bytecode writer library loaded")
(print "Testing u32-to-bytes:")
(print (u32-to-bytes 0))
(print (u32-to-bytes 255))
(print (u32-to-bytes 256))
(print (u32-to-bytes 65536))

(print "Testing string serialization:")
(print (serialize-string "hello"))
