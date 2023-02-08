pragma circom 2.1.3;

template Multiplier() {

   signal input a;
   signal input b;
   signal output c;

   c <== a * b;


   // Check that the `a` and `b` inputs are not equal
   signal inv_diff;

   var diff = a - b;
   inv_diff <-- 1 / diff;
   1 === diff * inv_diff;

   component bounded[2];

   bounded[0] = Num2Bits(255);
   bounded[0].num <== a;

   bounded[1] = Num2Bits(255);
   bounded[1].num <== b;

}

template Num2Bits(n) {

    signal input num;
    signal output bits[n];
    var mul = 1;
    var total = 0;

    for (var i = 0; i < n; i++) {
       bits[i] <-- (num >> i) & 1;
       bits[i] * (bits[i] - 1) === 0;

       total += bits[i] * mul;
       mul *= 2;
    }

    // Total will be expanded to a expression, that will then be converted to a set of constraints
    total === num;
}

component main = Multiplier();