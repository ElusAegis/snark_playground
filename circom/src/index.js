const snarkjs = require("snarkjs");
const fs = require("fs");

const prompt = require("prompt-sync")({ sigint: true });

const BUILD_DIR = "build";

async function run() {
    const circuitName = "private_multiplication";

    // Ask the user to input the private inputs from stdin
    const a = parseInt(prompt("Enter a: "));
    const b = parseInt(prompt("Enter b: "));

    // Try to do the groth16 proof, if it fails, print an error message and exit
    let proof, publicSignals;
    try {
        let res = await snarkjs.groth16.fullProve({ a, b }, `${BUILD_DIR}/${circuitName}_js/${circuitName}.wasm`, `${BUILD_DIR}/${circuitName}_final.zkey`);
        proof = res.proof;
        publicSignals = res.publicSignals;
    } catch (e) {
        console.log("Incorrect inputs to the circuit");
        return;
    }

    console.log("Proof: ");
    console.log(JSON.stringify(proof, null, 1));

    console.log("Public signals: c = " + publicSignals);

    // Ask the user if they want to verify the proof
    const verify = prompt("Verify? (y/n) ");
    if (verify !== "y") {
        return;
    }

    const vKey = JSON.parse(fs.readFileSync(`${BUILD_DIR}/verification_key.json`));

    const res = await snarkjs.groth16.verify(vKey, publicSignals, proof);

    if (res === true) {
        console.log("Verification OK");
    } else {
        console.log("Invalid proof");
    }

}

run().then(() => {
    process.exit(0);
});