; association lookup over quoted pair data
(define table '(("alpha" 20) ("beta" 22) ("gamma" 30)))
(defun assoc (k xs)
  (if (null? xs) nil
    (if (= (car (car xs)) k) (car xs) (assoc k (cdr xs)))))
(defun val (k xs) (car (cdr (assoc k xs))))
(print (assoc "beta" table))
(+ (val "alpha" table) (val "beta" table))
