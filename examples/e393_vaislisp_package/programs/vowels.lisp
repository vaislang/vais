; count vowels by scanning bytes with str-byte inside a let frame;
; vowel? is an or-chain and the counting if has no else branch
(defun vowel? (b)
  (or (= b 97) (= b 101) (= b 105) (= b 111) (= b 117)))
(defun count-vowels (s)
  (let ((i 0) (n (str-len s)) (acc 0))
    (while (< i n)
      (if (vowel? (str-byte s i)) (set acc (+ acc 1)))
      (set i (+ i 1)))
    acc))
(print (count-vowels "vaislisp dogfooding session"))
(+ (count-vowels "aeiou") 37)
