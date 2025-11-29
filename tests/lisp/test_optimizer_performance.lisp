;; Performance test: demonstrate optimizer reduces instruction count
;; EXPECT: 100

;; Without optimization, this would generate many Push + Add instructions
;; With optimization, constants are folded at compile time

(defun long-constant-chain ()
  ;; Each operation can be folded
  (+ 1 2 3 4 5 6 7 8 9 10))

;; Deeply nested constant expressions
(defun nested-constants ()
  ;; All these constants get folded
  (+ (* 2 3)
     (- 10 5)
     (/ 20 4)))

;; Float operations that benefit from folding
(defun float-chain ()
  ;; Multiple float operations folded
  (+ (* 2.5 4.0)    ; 10.0
     (/ 15.0 3.0)   ; 5.0
     (* 1.5 2.0)))  ; 3.0  -> total 18.0

;; Mixed operations
(defun mixed-chain ()
  ;; Type coercion happens at compile time
  (+ 10             ; int
     (* 5 2.0)      ; int * float -> 10.0
     (/ 30.0 3)))   ; float / int -> 10.0

;; The optimizer will fold (+ 10 20 30 40)
(+ 10 20 30 40)
