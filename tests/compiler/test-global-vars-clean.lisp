; Clean test of global variables with comments!

; Constants
(def PI 3)
(def GREETING "Hello")

; Variables
(def age 35)
(def name "henrique")

; Test basic access
(print "=== Basic Access ===")
(print PI)
(print age)
(print name)

; Test in expressions
(print "=== In Expressions ===")
(print (+ PI 1))
(print (+ age 10))

; Test in functions
(defun greet
  ((person)
    (let ((x GREETING))
      (let ((y person))
        (print x)
        (print y)))))

(print "=== From Functions ===")
(greet name)

; Test closure access
(defun make-adder
  ((n) (lambda (x) (+ x n))))

(let ((add-pi (make-adder PI)))
  (print "=== From Closures ===")
  (print (add-pi 10)))
