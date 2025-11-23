(defun countdown (n)
  (if (<= n 0)
    (print "done")
    (countdown (- n 1))))

(print "Testing countdown with 100000 iterations...")
(countdown 100000)
(print "Success! No stack overflow")

(defun fact-tail (n acc)
  (if (<= n 0)
    acc
    (fact-tail (- n 1) (* n acc))))

(print "Testing tail-recursive factorial...")
(print (fact-tail 10 1))

(defun sum (n acc)
  (if (<= n 0)
    acc
    (sum (- n 1) (+ n acc))))

(print "Testing tail-recursive sum 1..10000...")
(print (sum 10000 0))

(defun even? (n)
  (if (== n 0)
    true
    (odd? (- n 1))))

(defun odd? (n)
  (if (== n 0)
    false
    (even? (- n 1))))

(print "Testing mutual tail recursion...")
(print (even? 10000))
(print (odd? 10000))

(print "All TCO tests passed!")
