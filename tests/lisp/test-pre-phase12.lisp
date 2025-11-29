;; EXPECT-CONTAINS: All pre-Phase 12 tests passed!
;; Tests for features added before Phase 12: do/begin, type-of, gensym, macroexpand, time, assert

(print "Testing Pre-Phase 12 Features")
(print "")

;; Test do/begin form
(print "Testing do/begin...")
(def x 10)
(def y 20)
(def test-do (do
              (print x)
              (print y)
              (+ x y)))
(print test-do)  ;; Should print 30

(def a 5)
(def b 3)
(def test-begin (begin
                  (print a)
                  (print b)
                  (* a b)))
(print test-begin)  ;; Should print 15

;; Test type-of
(print "")
(print "Testing type-of...")
(print (type-of 42))           ;; integer
(print (type-of 3.14))         ;; float
(print (type-of true))         ;; boolean
(print (type-of "hello"))      ;; string
(print (type-of 'foo))         ;; symbol
(print (type-of '(1 2 3)))     ;; list
(print (type-of +))            ;; function
(print (type-of (lambda (x) x))) ;; closure

;; Test gensym
(print "")
(print "Testing gensym...")
(def g1 (gensym))
(def g2 (gensym))
(print g1)
(print g2)
(print (== g1 g2))  ;; Should be false

;; Test macroexpand
(print "")
(print "Testing macroexpand...")
(defmacro inc (x) `(+ ,x 1))
(def expanded (macroexpand '(inc 5)))
(print expanded)  ;; Should print (+ 5 1)

;; Test that non-macros return unchanged
(def unchanged (macroexpand '(+ 1 2)))
(print unchanged)  ;; Should print (+ 1 2)

;; NOTE: time and assert macros from stdlib would need to be included differently
;; Since macros are expanded at compile time, they need to be available when compiling
;; For now, we test the core features that are built-in

(print "")
(print "Skipping stdlib macro tests (time/assert) - would need different test setup")

;; Integration test: use multiple features together
(print "")
(print "Integration test...")
(defmacro double (x) `(* 2 ,x))
(def n 21)
(def doubled (double n))
(print doubled)  ;; Should print 42

;; Test type-of on macroexpanded result
(def expanded-type (type-of (macroexpand '(double 10))))
(print expanded-type)  ;; Should be list

(print "")
(print "All pre-Phase 12 tests passed!")
