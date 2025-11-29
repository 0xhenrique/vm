;; ============================================================
;; EXPERIMENTAL FEATURE TESTS
;; Testing the boundaries of our Lisp implementation
;; These tests help identify missing features and pain points
;; ============================================================

(print "=== EXPERIMENTAL FEATURE TESTS ===")
(print "")

;; ============================================================
;; SECTION 1: Basic Sequential Collections
;; ============================================================
(print "--- Section 1: Sequential Collections ---")

;; Test: map (sequential version - currently missing?)
(print "Test: map function")
;; (def mapped (map (lambda (x) (* x 2)) '(1 2 3 4 5)))
;; (print mapped)
(print "SKIPPED: map not implemented yet")

;; Test: filter (sequential version - currently missing?)
(print "Test: filter function")
;; (def filtered (filter (lambda (x) (> x 3)) '(1 2 3 4 5)))
;; (print filtered)
(print "SKIPPED: filter not implemented yet")

;; Test: reduce (sequential version - currently missing?)
(print "Test: reduce function")
;; (def sum (reduce (lambda (a b) (+ a b)) 0 '(1 2 3 4 5)))
;; (print sum)
(print "SKIPPED: reduce not implemented yet")

;; Test: range function
(print "Test: range function")
;; (def nums (range 1 10))
;; (print nums)
(print "SKIPPED: range not implemented yet")

;; Test: reverse list
(print "Test: reverse function")
;; (def reversed (reverse '(1 2 3 4 5)))
;; (print reversed)
(print "SKIPPED: reverse not implemented yet")

;; Test: take/drop
(print "Test: take/drop functions")
;; (print (take 3 '(1 2 3 4 5)))
;; (print (drop 2 '(1 2 3 4 5)))
(print "SKIPPED: take/drop not implemented yet")

(print "")

;; ============================================================
;; SECTION 2: Higher-Order Functions
;; ============================================================
(print "--- Section 2: Higher-Order Functions ---")

;; Test: compose
(print "Test: compose function")
;; (def add1 (lambda (x) (+ x 1)))
;; (def mul2 (lambda (x) (* x 2)))
;; (def composed (compose mul2 add1))
;; (print (composed 5))  ; Should be 12
(print "SKIPPED: compose not implemented yet")

;; Test: apply
(print "Test: apply function")
;; (def sum-all (lambda (& args) (reduce + 0 args)))
;; (print (apply sum-all '(1 2 3 4 5)))
(print "SKIPPED: apply not implemented yet")

;; Test: partial application
(print "Test: partial function")
;; (def add (lambda (a b) (+ a b)))
;; (def add5 (partial add 5))
;; (print (add5 10))  ; Should be 15
(print "SKIPPED: partial not implemented yet")

(print "")

;; ============================================================
;; SECTION 3: String Manipulation
;; ============================================================
(print "--- Section 3: String Manipulation ---")

;; Test: string functions we have
(print "Test: existing string functions")
(print (string-length "hello"))
(print (string-append "hello" " world"))
(print (substring "hello world" 0 5))

;; Test: string case conversion
(print "Test: string case conversion")
;; (print (string-upcase "hello"))
;; (print (string-downcase "WORLD"))
(print "SKIPPED: string-upcase/downcase not implemented yet")

;; Test: string predicates
(print "Test: string predicates")
;; (print (string-starts-with? "hello" "hel"))
;; (print (string-ends-with? "hello" "lo"))
;; (print (string-contains? "hello world" "wo"))
(print "SKIPPED: string predicates not implemented yet")

;; Test: string formatting
(print "Test: format/sprintf")
;; (print (format "Hello {}, you are {} years old" "Alice" 30))
(print "SKIPPED: format not implemented yet")

(print "")

;; ============================================================
;; SECTION 4: Math Functions
;; ============================================================
(print "--- Section 4: Math Functions ---")

;; Test: basic math we have
(print "Test: existing math")
(print (+ 1 2 3 4 5))
(print (* 2 3 4))
(print (- 10 3))
(print (/ 10 2))

;; Test: advanced math
(print "Test: advanced math functions")
;; (print (sqrt 16))
;; (print (pow 2 10))
;; (print (abs -5))
;; (print (floor 3.7))
;; (print (ceil 3.2))
;; (print (round 3.5))
(print "SKIPPED: sqrt/pow/abs/floor/ceil/round not implemented yet")

;; Test: trigonometry
(print "Test: trig functions")
;; (print (sin 0))
;; (print (cos 0))
;; (print (tan 0))
(print "SKIPPED: trig functions not implemented yet")

;; Test: min/max
(print "Test: min/max")
;; (print (min 1 5 3 9 2))
;; (print (max 1 5 3 9 2))
(print "SKIPPED: min/max not implemented yet")

(print "")

;; ============================================================
;; SECTION 5: Error Handling
;; ============================================================
(print "--- Section 5: Error Handling ---")

;; Test: try/catch
(print "Test: try/catch")
;; (def result (try
;;   (/ 10 0)
;;   (catch e
;;     (print "Caught error:")
;;     (print e)
;;     0)))
(print "SKIPPED: try/catch not implemented yet")

;; Test: throw
(print "Test: throw/raise")
;; (defun risky ()
;;   (throw "Something went wrong"))
(print "SKIPPED: throw not implemented yet")

(print "")

;; ============================================================
;; SECTION 6: Data Structures
;; ============================================================
(print "--- Section 6: Data Structures ---")

;; Test: hashmap functions we have
(print "Test: existing hashmap functions")
;; Note: hashmap constructor not available, cannot test properly
;; (def my-map (hashmap))
;; (def my-map2 (hashmap-set my-map "name" "Alice"))
;; (def my-map3 (hashmap-set my-map2 "age" 30))
;; (print (hashmap-get my-map3 "name"))
;; (print (hashmap-contains-key? my-map3 "name"))
;; (print (hashmap-keys my-map3))
;; (print (hashmap-values my-map3))
(print "SKIPPED: hashmap constructor not available")

;; Test: hash-map iteration
(print "Test: hash-map-each")
;; (hash-map-each my-map (lambda (k v)
;;   (print (string-append k ": " (number->string v)))))
(print "SKIPPED: hash-map-each not implemented yet")

;; Test: vector functions we have
(print "Test: existing vector functions")
(def my-vec (vector 1 2 3 4 5))
(print (vector-ref my-vec 2))
(print (vector-length my-vec))

;; Test: vector mutation
(print "Test: vector-set!")
;; (def my-vec (vector 1 2 3))
;; (vector-set! my-vec 1 99)
;; (print my-vec)
(print "SKIPPED: vector-set! not implemented yet")

;; Test: vector-push/pop
(print "Test: vector-push!/vector-pop!")
;; (def my-vec (vector 1 2 3))
;; (vector-push! my-vec 4)
;; (print my-vec)
;; (def popped (vector-pop! my-vec))
;; (print popped)
(print "SKIPPED: vector-push!/pop! not implemented yet")

;; Test: sets
(print "Test: set data structure")
;; (def my-set (set 1 2 3 4 5))
;; (print (set-contains? my-set 3))
;; (def my-set (set-add my-set 6))
;; (def my-set (set-remove my-set 2))
(print "SKIPPED: set not implemented yet")

(print "")

;; ============================================================
;; SECTION 7: I/O Operations
;; ============================================================
(print "--- Section 7: I/O Operations ---")

;; Test: file reading (we have read-file)
(print "Test: file reading")
;; (def content (read-file "test.txt"))
;; (print content)
(print "SKIPPED: read-file exists but needs test file")

;; Test: file writing
(print "Test: write-file")
;; (write-file "output.txt" "Hello, World!")
(print "SKIPPED: write-file not implemented yet")

;; Test: file existence check
(print "Test: file-exists?")
;; (print (file-exists? "test.txt"))
(print "SKIPPED: file-exists? not implemented yet")

;; Test: directory operations
(print "Test: directory operations")
;; (print (list-dir "."))
;; (mkdir "test-dir")
(print "SKIPPED: directory ops not implemented yet")

;; Test: stdin/stdout
(print "Test: read-line")
;; (print "Enter your name: ")
;; (def name (read-line))
;; (print (string-append "Hello, " name))
(print "SKIPPED: read-line not implemented yet")

(print "")

;; ============================================================
;; SECTION 8: Control Flow
;; ============================================================
(print "--- Section 8: Control Flow ---")

;; Test: when/unless
(print "Test: when/unless")
;; (when (> 5 3) (print "5 is greater than 3"))
;; (unless (< 5 3) (print "5 is not less than 3"))
(print "SKIPPED: when/unless not implemented yet (can use if)")

;; Test: cond
(print "Test: cond")
;; (def x 5)
;; (cond
;;   ((< x 0) (print "negative"))
;;   ((== x 0) (print "zero"))
;;   ((> x 0) (print "positive")))
(print "SKIPPED: cond not implemented yet (can use nested if)")

;; Test: case/switch
(print "Test: case/switch")
;; (def x 2)
;; (case x
;;   (1 (print "one"))
;;   (2 (print "two"))
;;   (3 (print "three"))
;;   (default (print "other")))
(print "SKIPPED: case not implemented yet")

;; Test: and/or (we might have these?)
(print "Test: and/or operators")
;; (print (and true true false))
;; (print (or false false true))
(print "SKIPPED: and/or not implemented yet")

(print "")

;; ============================================================
;; SECTION 9: Functional Patterns
;; ============================================================
(print "--- Section 9: Functional Patterns ---")

;; Test: let bindings (do we have let?)
(print "Test: let bindings")
;; (let ((x 5)
;;       (y 10))
;;   (print (+ x y)))
(print "SKIPPED: let not implemented yet (use def)")

;; Test: destructuring
(print "Test: destructuring")
;; (def (first second & rest) '(1 2 3 4 5))
;; (print first)
;; (print second)
;; (print rest)
(print "SKIPPED: destructuring not implemented yet")

;; Test: pattern matching
(print "Test: pattern matching")
;; (match '(1 2 3)
;;   (() (print "empty"))
;;   ((x) (print "one element"))
;;   ((x y & rest) (print "two or more")))
(print "SKIPPED: pattern matching not implemented yet")

(print "")

;; ============================================================
;; SECTION 10: Metaprogramming
;; ============================================================
(print "--- Section 10: Metaprogramming ---")

;; Test: eval (do we have eval?)
(print "Test: eval")
;; (def code '(+ 1 2 3))
;; (print (eval code))
(print "SKIPPED: eval not implemented yet")

;; Test: quote (we have ' which is quote)
(print "Test: quote")
(def quoted '(+ 1 2 3))
(print quoted)
(print "OK: quote works with '")

;; Test: quasiquote/unquote
(print "Test: quasiquote/unquote")
;; (def x 5)
;; (def code `(+ 1 2 ,x))
;; (print code)  ; Should be (+ 1 2 5)
(print "SKIPPED: quasiquote/unquote not implemented yet")

;; Test: macros (we have macroexpand)
(print "Test: macro definition")
;; (defmacro unless (condition & body)
;;   `(if (not ,condition)
;;     (begin ,@body)))
(print "SKIPPED: defmacro not fully implemented yet")

(print "")

;; ============================================================
;; SECTION 11: Type System
;; ============================================================
(print "--- Section 11: Type System ---")

;; Test: type predicates we have
(print "Test: existing type predicates")
(print (integer? 42))
(print (float? 3.14))
(print (string? "hello"))
(print (list? '(1 2 3)))
(print (function? +))

;; Test: type conversion
(print "Test: type conversions we have")
(print (number->string 42))
(print (string->number "42"))
(print (symbol->string 'foo))
(print (string->symbol "foo"))

(print "")

;; ============================================================
;; SECTION 12: Concurrency (Future)
;; ============================================================
(print "--- Section 12: Concurrency ---")

;; Test: threads
(print "Test: spawn thread")
;; (def thread (spawn (lambda () (print "Hello from thread"))))
;; (thread-join thread)
(print "SKIPPED: threads not implemented yet")

;; Test: channels
(print "Test: channels")
;; (def ch (channel))
;; (channel-send ch 42)
;; (print (channel-recv ch))
(print "SKIPPED: channels not implemented yet")

;; Test: async/await
(print "Test: async/await")
;; (def result (await (async (+ 1 2))))
(print "SKIPPED: async not implemented yet")

(print "")

;; ============================================================
;; SECTION 13: Memory & Performance
;; ============================================================
(print "--- Section 13: Memory & Performance ---")

;; Test: memoization
(print "Test: memoize")
;; (def fib-memo (memoize fib))
(print "SKIPPED: memoize not implemented yet")

;; Test: lazy evaluation
(print "Test: lazy sequences")
;; (def lazy-nums (lazy-seq (range 0 1000000)))
;; (print (take 10 lazy-nums))
(print "SKIPPED: lazy sequences not implemented yet")

(print "")

(print "=== EXPERIMENTAL TESTS COMPLETE ===")
(print "See output above for missing features")
