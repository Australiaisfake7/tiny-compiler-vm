// 12 - Classic algorithms
//
// Loops, arrays, functions and nullable results used together.

// ---- Sieve of Eratosthenes: all primes below 50 ---------------------------------------------------
bool[50] composite = [false: 50];
for (int i = 2; i < 50; i = i + 1) {
    if (!composite[i]) {
        print i;
        for (int multiple = i * i; multiple < 50; multiple = multiple + i) {
            composite[multiple] = true;
        }
    }
}

// ---- Bubble sort -----------------------------------------------------------------------------------
fun void sort(int[8] values) {
    for (int pass = 0; pass < 7; pass = pass + 1) {
        for (int j = 0; j < 7 - pass; j = j + 1) {
            if (values[j] > values[j + 1]) {
                int swap = values[j];
                values[j] = values[j + 1];
                values[j + 1] = swap;
            }
        }
    }
}

int[8] numbers = [42; 7; 19; 3; 88; 23; 5; 61];
print numbers;
sort(numbers);
print numbers;

// ---- Binary search on the sorted array: the index, or null when absent ---------------------------------
fun int? binarySearch(int[8] sorted; int target) {
    int low = 0;
    int high = 7;
    while (low <= high) {
        int middle = (low + high) / 2;
        if (sorted[middle] == target) { return middle; }
        if (sorted[middle] < target) { low = middle + 1; }
        else { high = middle - 1; }
    }
    return null;
}

print binarySearch(numbers; 23);
print binarySearch(numbers; 24);

// ---- Largest value and reversal ------------------------------------------------------------------------
fun int largest(int[8] values) {
    int best = values[0];
    for (int i = 1; i < 8; i = i + 1) {
        if (values[i] > best) { best = values[i]; }
    }
    return best;
}

fun void reverse(int[8] values) {
    for (int i = 0; i < 4; i = i + 1) {
        int other = 7 - i;
        int swap = values[i];
        values[i] = values[other];
        values[other] = swap;
    }
}

print largest(numbers);
reverse(numbers);
print numbers;

// ---- Collatz: how many steps until 1? ----------------------------------------------------------------------
fun int collatzSteps(int start) {
    int n = start;
    int steps = 0;
    while (n != 1) {
        if (n - n / 2 * 2 == 0) { n = n / 2; }
        else { n = 3 * n + 1; }
        steps = steps + 1;
    }
    return steps;
}

print collatzSteps(6);
print collatzSteps(27);

// ---- Fibonacci without recursion -------------------------------------------------------------------------------
int previous = 0;
int current = 1;
for (int i = 0; i < 10; i = i + 1) {
    print previous;
    int next = previous + current;
    previous = current;
    current = next;
}
