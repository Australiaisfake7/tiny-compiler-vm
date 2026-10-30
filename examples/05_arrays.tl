// 05 - Arrays
//
// Arrays have a fixed length that is part of their type: int[5] holds five ints.
// Indexing starts at 0 and every access is bounds checked.

// ---- Creating arrays -------------------------------------------------------------
int[5] primes = [2; 3; 5; 7; 11];       // elements are separated by semicolons
float[3] weights = [0.5; 1.25; 2.0];
string[2] words = [ "alpha"; "beta" ];
bool[4] flags = [true; false; false; true];

int[6] zeros = [0: 6];                  // [value: length] fills an array with one value

print primes;
print weights;
print words;
print zeros;

// ---- Reading and writing ---------------------------------------------------------
print primes[0];
print primes[4];

primes[0] = 100;
print primes[0];

// ---- Looping over an array ---------------------------------------------------------
int total = 0;
for (int i = 0; i < 5; i = i + 1) {
    total = total + primes[i];
}
print total;

// ---- Arrays of arrays ----------------------------------------------------------------
// The first dimension is the outermost: int[3][2] is an array of 3 rows,
// and each row is an int[2]. The first index picks the row.
int[3][2] grid = [[0: 2]: 3];
grid[0][0] = 1;
grid[1][1] = 5;
grid[2][0] = 9;
print grid;
print grid[1][1];

for (int row = 0; row < 3; row = row + 1) {
    for (int col = 0; col < 2; col = col + 1) {
        print grid[row][col];
    }
}

// ---- Arrays are shared, not copied ---------------------------------------------------
int[3] original = [1; 2; 3];
int[3] alias = original;
alias[0] = 99;
print original[0];                      // 99: both names refer to the same array

// ---- Equality compares the contents ---------------------------------------------------
int[3] a = [1; 2; 3];
int[3] b = [1; 2; 3];
print a == b;
b[2] = 4;
print a == b;

// ---- Passing arrays to functions and returning them ------------------------------------
fun void doubleAll(int[3] values) {
    for (int i = 0; i < 3; i = i + 1) {
        values[i] = values[i] * 2;
    }
}

fun int[3] countUp() {
    return [1; 2; 3];
}

int[3] nums = countUp();
doubleAll(nums);                        // changes the caller's array
print nums;

// ---- Bounds are checked ----------------------------------------------------------------
// Uncommenting these lines stops the program with:
//   runtime error: index 5 is out of bounds for an array of length 3
// int outside = 5;
// print nums[outside];
