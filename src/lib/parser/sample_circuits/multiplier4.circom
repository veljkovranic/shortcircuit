pragma circom 2.1.6;
include "circomlib/circuits/comparators.circom";

template Multiplier4() {
     signal input in1;
     signal input in2;
     signal input in3;
     signal input in4;
     signal input mult;
     signal output out;
     signal tmp1 <== in1*in2;
     signal tmp2 <== in3*in4;
     component isEqual = IsEqual();
     isEqual.in[0] <== mult;
     isEqual.in[1] <== tmp1*tmp2;
     out <== isEqual.out;
}


component main = Multiplier4();


/* INPUT = {
    "in1": "5",
    "in2": "3",
    "in3": "5",
    "in4": "7",
    "mult": "525"
} */