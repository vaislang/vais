; fizzbuzz over 1..20 — modulo built from integer division
(defun mod (a b) (- a (* (/ a b) b)))
(define n 1)
(while (< n 21)
  (if (= (mod n 15) 0) (print "FizzBuzz")
    (if (= (mod n 3) 0) (print "Fizz")
      (if (= (mod n 5) 0) (print "Buzz")
        (print n))))
  (set n (+ n 1)))
42
