;; FFI (Foreign Function Interface) Tests
;; Tests for calling C functions from Lisp

;; ==================== Basic Pointer Operations ====================

;; Test null pointer creation
(print "Testing ffi-null...")
(def null-ptr (ffi-null))
(print (if (pointer? null-ptr) "PASS: ffi-null returns pointer" "FAIL: ffi-null should return pointer"))
(print (if (ffi-null? null-ptr) "PASS: ffi-null? detects null" "FAIL: ffi-null? should detect null"))

;; Test pointer? predicate
(print "Testing pointer? predicate...")
(print (if (pointer? (ffi-null)) "PASS: pointer? true for null" "FAIL"))
(print (if (not (pointer? 42)) "PASS: pointer? false for int" "FAIL"))
(print (if (not (pointer? "string")) "PASS: pointer? false for string" "FAIL"))

;; ==================== Memory Allocation ====================

(print "Testing memory allocation...")
(def ptr (ffi-allocate 1024))
(print (if (not (ffi-null? ptr)) "PASS: ffi-allocate returns non-null" "FAIL"))
(def freed (ffi-free ptr))
(print (if freed "PASS: ffi-free returns true" "FAIL"))

;; ==================== Memory Read/Write ====================

(print "Testing integer read/write...")
(def int-ptr (ffi-allocate 8))
(ffi-write-int int-ptr 12345)
(def read-val (ffi-read-int int-ptr))
(print (if (== read-val 12345) "PASS: read/write int works" "FAIL"))
(ffi-free int-ptr)

(print "Testing float read/write...")
(def float-ptr (ffi-allocate 8))
(ffi-write-float float-ptr 3.14159)
(def read-float (ffi-read-float float-ptr))
;; Check that it's close to 3.14159 (within tolerance)
(print (if (< (abs (- read-float 3.14159)) 0.0001) "PASS: read/write float works" "FAIL"))
(ffi-free float-ptr)

(print "Testing byte read/write...")
(def byte-ptr (ffi-allocate 1))
(ffi-write-byte byte-ptr 255)
(def read-byte (ffi-read-byte byte-ptr))
(print (if (== read-byte 255) "PASS: read/write byte works" "FAIL"))
(ffi-free byte-ptr)

;; ==================== String Conversion ====================

(print "Testing string conversion...")
(def str-ptr (ffi-string->pointer "hello world"))
(print (if (not (ffi-null? str-ptr)) "PASS: string->pointer non-null" "FAIL"))
(def str-back (ffi-pointer->string str-ptr))
(print (if (== str-back "hello world") "PASS: string roundtrip works" "FAIL"))
(ffi-free-string str-ptr)

;; ==================== Pointer Arithmetic ====================

(print "Testing pointer arithmetic...")
(def base (ffi-allocate 16))
(def offset (ffi-pointer+ base 8))
(def diff (- offset base))
(print (if (== diff 8) "PASS: pointer arithmetic works" "FAIL"))
(ffi-free base)

;; ==================== Library Loading and FFI Calls ====================

(print "Testing libc loading...")
;; Try to load libc
(def libc-result
    (if (file-exists? "/lib/x86_64-linux-gnu/libc.so.6")
        (ffi-load "/lib/x86_64-linux-gnu/libc.so.6")
        (if (file-exists? "/lib64/libc.so.6")
            (ffi-load "/lib64/libc.so.6")
            (ffi-load "libc.so.6"))))

(if (> libc-result 0)
    (do
        (print "PASS: libc loaded successfully")

        ;; Test strlen
        (print "Testing strlen via FFI...")
        (def strlen-ptr (ffi-symbol libc-result "strlen"))
        (def len (ffi-call strlen-ptr (:string) :int64 "hello"))
        (print (if (== len 5) "PASS: strlen works" (format "FAIL: expected 5, got {}" len)))

        ;; Test abs
        (print "Testing abs via FFI...")
        (def abs-ptr (ffi-symbol libc-result "abs"))
        (def abs-result (ffi-call abs-ptr (:int32) :int32 -42))
        (print (if (== abs-result 42) "PASS: abs works" (format "FAIL: expected 42, got {}" abs-result)))

        ;; Test getenv
        (print "Testing getenv via FFI...")
        (def getenv-ptr (ffi-symbol libc-result "getenv"))
        (def path-ptr (ffi-call getenv-ptr (:string) :pointer "PATH"))
        (print (if (not (ffi-null? path-ptr)) "PASS: getenv(PATH) returns non-null" "INFO: PATH not set (unusual)")))

    (print "INFO: Could not load libc, skipping libc-dependent tests"))

;; ==================== Multiple FFI Calls ====================

(if (> libc-result 0)
    (do
        (print "Testing multiple FFI calls...")
        (def strlen-ptr (ffi-symbol libc-result "strlen"))
        (def total (+ (ffi-call strlen-ptr (:string) :int64 "one")
                      (+ (ffi-call strlen-ptr (:string) :int64 "two")
                         (ffi-call strlen-ptr (:string) :int64 "three"))))
        (print (if (== total 11) "PASS: multiple strlen calls" (format "FAIL: expected 11, got {}" total))))
    nil)

;; ==================== FFI Wrapper Functions ====================

(if (> libc-result 0)
    (do
        (print "Testing FFI wrapper function...")

        ;; Define a nice wrapper for strlen
        (defun c-strlen (s)
            (let ((strlen-ptr (ffi-symbol libc-result "strlen")))
                (ffi-call strlen-ptr (:string) :int64 s)))

        (print (if (== (c-strlen "wrapper test") 12)
                   "PASS: wrapper function works"
                   "FAIL: wrapper function broken")))
    nil)

;; ==================== Done ====================

(print "")
(print "FFI tests completed!")
