;; Test optimizer constant folding with integers and floats
;; EXPECT: 42

;; Integer constant folding
;; (+ 40 2) should be folded to 42 at compile time
(defun test-int-folding ()
  (+ 40 2))

;; Float constant folding
;; (* 2.5 4.0) should be folded to 10.0 at compile time
(defun test-float-folding ()
  (* 2.5 4.0))

;; Mixed int/float constant folding
;; (+ 5 2.5) should be folded to 7.5 at compile time
(defun test-mixed-folding ()
  (+ 5 2.5))

;; Complex expression with multiple folds
;; (+ (* 2 3) (- 10 4)) = (+ 6 6) = 12
(defun test-complex-folding ()
  (+ (* 2 3) (- 10 4)))

;; Nested arithmetic
;; (* (+ 1 2) (- 5 1)) = (* 3 4) = 12
(defun test-nested-folding ()
  (* (+ 1 2) (- 5 1)))

;; Float division that gets folded
;; (/ 21.0 2.0) = 10.5
(defun test-float-div-folding ()
  (/ 21.0 2.0))

;; Comparison folding
;; (> 5 3) = #t at compile time
(defun test-comparison-folding ()
  (if (> 5 3)
      42
      0))

;; The main test just returns 42 to verify execution
(test-comparison-folding)
