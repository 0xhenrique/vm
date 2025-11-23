(defun map
  ((f '()) '())
  ((f (h . t)) (cons (f h) (map f t))))

(defun filter
  ((predicate '()) '())
  ((predicate (h . t))
    (if (predicate h)
      (cons h (filter predicate t))
      (filter predicate t))))

(defun reduce
  ((f acc '()) acc)
  ((f acc (h . t)) (reduce f (f acc h) t)))

(print (map (lambda (x) (* x 2)) '(1 2 3 4 5)))

(print (map (lambda (x) (+ x 1)) '(10 20 30)))

(print (map (lambda (x) x) '(1 2 3)))

(print (map (lambda (x) (* x x)) '(1 2 3 4)))

(print (map (lambda (x) 0) '(1 2 3)))

(print (map (lambda (x) (+ x 1)) '()))

(print (filter (lambda (x) (> x 2)) '(1 2 3 4 5)))

(print (filter (lambda (x) (< x 10)) '(5 15 8 20 3)))

(print (filter (lambda (x) true) '(1 2 3)))

(print (filter (lambda (x) false) '(1 2 3)))

(print (filter (lambda (x) (> x 0)) '()))

(print (reduce (lambda (acc x) (+ acc x)) 0 '(1 2 3 4 5)))

(print (reduce (lambda (acc x) (* acc x)) 1 '(1 2 3 4 5)))

(print (reduce (lambda (acc x) (cons x acc)) '() '(1 2 3)))

(print (reduce (lambda (acc x) (+ acc x)) 10 '(1 2 3)))

(print (reduce (lambda (acc x) (+ acc x)) 100 '()))

(print (filter (lambda (x) (> x 5))
               (map (lambda (x) (* x 2)) '(1 2 3 4 5))))

(print (reduce (lambda (acc x) (+ acc x))
               0
               (map (lambda (x) (* x x)) '(1 2 3 4))))

(print (reduce (lambda (acc x) (+ acc 1)) 0 '(a b c d e)))

(print (reduce (lambda (a b) (if (> a b) a b)) 0 '(3 7 2 9 4 1)))

(defun is-even (n)
  (if (< n 2)
    (if (< n 1)
      true
      false)
    (is-even (- n 2))))

(print (filter (lambda (n)
                 (if (< n 2)
                   (if (< n 1)
                     true
                     false)
                   (is-even (- n 2))))
               '(1 2 3 4 5 6 7 8)))

(print (reduce (lambda (acc x) (+ acc x))
               0
               (map (lambda (x) (* x x))
                    (filter (lambda (n)
                              (if (< n 2)
                                (if (< n 1)
                                  true
                                  false)
                                (is-even (- n 2))))
                            '(1 2 3 4 5 6 7 8)))))

(defun make-adder (n)
  (lambda (x) (+ x n)))

(print (map (make-adder 10) '(1 2 3)))

(defun make-predicate (threshold)
  (lambda (x) (> x threshold)))

(print (filter (make-predicate 5) '(3 6 2 8 4 9)))

(print (map (lambda (x)
              (reduce (lambda (a b) (+ a b))
                      0
                      '(1 2 3)))
            '(a b c)))
