### Script for generating the circuit and the witness
This script uses insecure random numbers. Do not use it for production.

The script generates the circuit and the witness for the private multiplication circuit.
It can optionally do the tau ceremony and generate the proving and verification keys.

#### Usage
```
./build.sh CIRCUIT_NAME [OPTIONS]
```

Where `CIRCUIT_NAME` is the name of the circuit to generate. 

#### Options
```
--curve CURVE_NAME
    The curve to use. Default: BN128
--constrain AMOUNT
    The amount of constraints to use. Accepts the power of 2. Default: 2^10
--clean 
    Clean the build directory before building
--snark SNARK_NAME
    The snark to use. Default: groth16
```

#### Example
```
./build.sh private_multiplication.circom --clean --constraints 11 --snark groth16
```