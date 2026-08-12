; higher-order map and fold over quoted data
(defun map (f xs) (if (null? xs) nil (cons (f (car xs)) (map f (cdr xs)))))
(defun fold (f acc xs) (if (null? xs) acc (fold f (f acc (car xs)) (cdr xs))))
(define squares (map (lambda (x) (* x x)) '(1 2 3 4)))
(print squares)
(+ (fold (lambda (a b) (+ a b)) 0 squares) 12)
