;; EXPECT: all-tests-passed
;; Test reflection features (function introspection)

;; Test 1: function-arity for regular functions
(defun add (a b) (+ a b))
(defun triple (a b c) (+ a (+ b c)))

(def arity-add (function-arity add))
(def arity-triple (function-arity triple))

;; Test 2: function-arity for variadic functions
(defun var-func (a . rest) (cons a rest))
(def arity-var (function-arity var-func))

;; Test 3: function-params for closures
(defun make-adder (x)
    (lambda (y) (+ x y)))

(def adder (make-adder 10))
(def params (function-params adder))

;; Test 4: closure-captured for closures
(def captured (closure-captured adder))

;; Test 5: function-name for named functions
(def fname (function-name add))

;; Verify results
(def test1 (== arity-add 2))
(def test2 (== arity-triple 3))
(def test3 (== arity-var -1))
(def test4 (== (car params) "y"))
(def test5 (== (string-length fname) 3)) ;; "add" has length 3
(def test6 (> (list-length captured) 0)) ;; Should have captured variables

;; Check if all tests passed
(def all-pass (if test1 (if test2 (if test3 (if test4 (if test5 test6 false) false) false) false) false))

(if all-pass
    (print "all-tests-passed")
    (print "some-tests-failed"))
