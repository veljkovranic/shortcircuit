pragma circom 2.1.6;

include "circomlib/circuits/mux1.circom";
include "circomlib/circuits/comparators.circom";

template Battleship(N) {
    signal input board[N][N];
    signal input ii;
    signal input jj;

    signal output answer;

    var isHit = 0;
    component isEqualCheck = IsEqual();

    component checkField[N][N];
    component isEqualI[N][N];
    component isEqualJ[N][N];

    for (var i = 0; i < N; i++) {
        for (var j = 0; j < N; j++) {
            isEqualI[i][j] = IsEqual();
            isEqualI[i][j].in[0] <== ii;
            isEqualI[i][j].in[1] <== i;

            isEqualJ[i][j] = IsEqual();
            isEqualJ[i][j].in[0] <== jj;
            isEqualJ[i][j].in[1] <== j;

            checkField[i][j] = Mux1();
            checkField[i][j].c[0] <== 0;
            checkField[i][j].c[1] <== board[i][j];
            checkField[i][j].s <== isEqualI[i][j].out * isEqualJ[i][j].out;
            isHit += checkField[i][j].out;
        }
    }
    answer <== isHit;
}


component main {public [board]} = Battleship(3);

/* INPUT = {
    "board": [
        ["0", "0", "1"], 
        ["0", "1", "1"], 
        ["1", "1", "0"]],
    "ii": "1",
    "jj": "1"
}
*/