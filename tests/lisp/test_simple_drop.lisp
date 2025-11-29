;; Simple drop test - baseline to verify basic list operations work
;; EXPECT: done

;; Test small list creation and dropping (list dropped when let scope ends)
(let ((small-list '(1 2 3 4 5)))
  (print (string-append "Small list length: " (number->string (list-length small-list)))))

;; Test medium list
(let ((medium-list (range 1 1001)))
  (print (string-append "Medium list length: " (number->string (list-length medium-list)))))

(print "done")
