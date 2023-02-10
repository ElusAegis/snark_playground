pragma circom 2.1.3;

template Multiplier() {

   signal input a;
   signal input b;
   signal output c;

   c <== a * b;

   component check_greater_or_equal_b_a = CheckGreaterOrEqual(255);
   check_greater_or_equal_b_a.a <== b;
   check_greater_or_equal_b_a.b <== a;

   // Check that the `c` and `b` inputs are not equal
   signal inv_diff;

   var diff = c - b;
   inv_diff <-- 1 / diff;
   // Check if there exists and inverse of the difference, this proves that the difference is not 0
   1 === diff * inv_diff;

   // Check that the `c` is greater or equal to `b`
   component check_greater_c_b = CheckGreaterOrEqual(255);
   check_greater_c_b.a <== c;
   check_greater_c_b.b <== b;

}

// Checks that a is greater or equal to b
template CheckGreaterOrEqual(nsize) {

    signal input a;
    signal input b;

    component a_bits_c = Num2Bits(nsize);
    a_bits_c.num <== a;
    signal a_bits[nsize];
    a_bits <== a_bits_c.bits;

    component b_bits_c = Num2Bits(nsize);
    b_bits_c.num <== b;
    signal b_bits[nsize];
    b_bits <== b_bits_c.bits;

    // Once we reach the first difference, we change this to 0 to avoid any further constraints
    signal finished[nsize + 1];

    // Set the finished to 1
    finished[nsize] <== 1;

    for (var i = nsize - 1; i >= 0; i--) {
        // Check if finished before or on this bit
        finished[i] <== finished[i + 1] * (1 - (a_bits[i] - b_bits[i]));

        // If a bit is bigger than b bit, then we are done and finished[i] is 0
        // Otherwise, they should be equal and we continue
        // Together, this means that the equation below should be 0
        finished[i] * (a_bits[i] - b_bits[i]) === 0;
    }

    // Enforce that last finish is 0 - meaning that a is bigger than b
    finished[0] === 0;
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