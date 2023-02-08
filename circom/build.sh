#!/bin/bash

# If any of the actions fail, exit the script
set -e


### Read the script arguments
# The script has one mandatory argument - the circuits name
# The script tree two optional arguments:
#   --curve <curve name> - the curve name, default is bn128
#   --constraints <amount of CONSTRAINTSs> - the amount of CONSTRAINTSs, default is 2^10
#   --clean - clean the build directory before building and redo the tau ceremony
#   --snark <snark system> - the snark system to use, default is plonk

# Check that there is at least one argument
# Read the circuits name
if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <circuit name> [--curve <curve name>] [--constraints <amount of CONSTRAINTSs>] [--clean] [--snark <snark system>]" >&2
  exit 1
fi
CIRCUIT=$1
# Remove the `.circom` extension from the circuits name
CIRCUIT=${CIRCUIT%.circom}
shift 1

# Check if the script `--clean` argument is set or the build directory does not exist
# If so, clean the build directory and redo the tau ceremony
if [ "$1" = "--clean" ] || [ ! -d "build" ]; then
  shift 1

  # Clean the build directory
  rm -rf build
  mkdir build
  cd build

  # Check if the script `--curve` argument is set
  # If so, read the next argument as the curve name
  # If not, use the default curve name
  if [ "$1" = "--curve" ]; then
    CURVE=$2
    shift 2
  else
    CURVE="bn128"
  fi

  # Check if the script `--constraints` argument is set
  # If so, read the next argument as the amount of CONSTRAINTSs (power of 2), max 2^28
  # If not, use the default amount of CONSTRAINTSs (2^15) and print a warning
  if [ "$1" = "--constraints" ]; then
    CONSTRAINTS=$2
    shift 2
  else
    CONSTRAINTS=10
    echo "WARNING: No CONSTRAINTS amount specified, using default value of 2^15"
  fi


  # Do a tau ceremony to generate the trusted setup
  # We do this in a insecure way, by using dummy randomness

  snarkjs powersoftau new $CURVE $CONSTRAINTS pot12_0000.ptau -v
  snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v -e="dummy randomness"
  snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v
else
  # If the script `--clean` argument is not set, just go to the build directory
  cd build
fi

# Check if the script `--snark` argument is set
# If so, read the next argument as the snark system to use
# If not, use the default snark system
if [ "$1" = "--snark" ]; then
  SNARK=$2
  shift 2
else
  SNARK="plonk"
fi


### Generate the circuits

# Generate the circuits R1CS and WASM files
circom "../circuits/$CIRCUIT.circom" --r1cs --wasm --sym

# Print the circuits information
snarkjs r1cs info "$CIRCUIT.r1cs"


# If groth16 is used, do the circuits setup
if [ "$SNARK" = "groth16" ]; then
  snarkjs groth16 setup $CIRCUIT.r1cs pot12_final.ptau $CIRCUIT"_"0000.zkey
  snarkjs zkey contribute $CIRCUIT"_"0000.zkey $CIRCUIT"_"final.zkey --name="Second contribution Name" -v -e="dummy randomness"
else
  # If plonk is used, do the circuits setup
  snarkjs plonk setup $CIRCUIT.r1cs pot12_final.ptau $CIRCUIT"_"final.zkey
fi


snarkjs zkey export verificationkey $CIRCUIT"_"final.zkey verification_key.json



# Ask the user to confirm that they have placed all of the input signals into the `inputs.json` file
read -p "Have you placed all of the input signals into the inputs.json file? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    exit 1
fi



node $CIRCUIT"_js/"generate_witness.js $CIRCUIT"_js/"$CIRCUIT.wasm ../input.json witness.wtns
snarkjs $SNARK prove $CIRCUIT"_"final.zkey witness.wtns ../$SNARK"_"proof.json ../public.json

# Print that the proof is generated
echo "Proof generated"

