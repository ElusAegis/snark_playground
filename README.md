# Project Collection

### 1. Private Multiplication

This is a simple example of how to create a private multiplication application. The objective is to create a circuit in multiple languages and DSL and compare the experience.

This is a particular simple circuit, but it is a good starting point for more complex circuits.


#### The constraints are as follows:

1. The circuit gets two private inputs (witness) and multiplies them. The result is then revealed by the prover (public input).
2. The circuit checks that the two inputs are smaller than the multiplication result, which itself is less than 2^255. This removes trivial solutions, such as `1 * n = n`, as well as solutions that exploit modular arithmetics, such as `a * b = n = -a * -b`.
   3. The circuit checks that the first input is smaller or equal to the second input. This removes trivial solution permutations, such as `a * b = n = b * a`.

#### Future work

This circuit prompts the user to give any factorization of `C`. This can be used to build a game where users are prompted to submit factorizations, only new factorizations are accepted, and each subsequent factorization, as it is harder to find, is worth more points.


