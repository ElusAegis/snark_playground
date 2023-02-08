# Private Multiplication

This is a simple example of how to create a private multiplication application. The objective is to create a circuit in multiple languages and DSL and compare the experience.

This is a particular simple circuit, but it is a good starting point for more complex circuits.


## The constraints are as follows:

1. The circuit gets two private inputs (witness) and multiplies them. The result is then revealed by the prover (public input).
2. Additionally, the circuit checks that the two inputs are smaller than the multiplication result, which itself is less than 2^32.
3. Finally, the circuit checks that the two inputs are not equal.


