; fizzbuzz over 1..20 — builtin mod, cond dispatch
(define n 1)
(while (< n 21)
  (cond ((= (mod n 15) 0) (print "FizzBuzz"))
        ((= (mod n 3) 0) (print "Fizz"))
        ((= (mod n 5) 0) (print "Buzz"))
        (else (print n)))
  (set n (+ n 1)))
42
