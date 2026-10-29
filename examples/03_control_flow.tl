// 03 - Control flow: if, while, for, break, continue

// ---- if / else if / else ----------------------------------------------------
int temperature = 23;

if (temperature < 0) {
    print "freezing";
} else if (temperature < 15) {
    print "cold";
} else if (temperature < 25) {
    print "pleasant";
} else {
    print "hot";
}

// Conditions must be bool: `if (1) {...}` is a compile error.

// ---- while ----------------------------------------------------------------
int n = 1;
while (n <= 100) {
    n = n * 2;
}
print n;

// ---- for ------------------------------------------------------------------
// for (initializer; condition; update) { ... }
for (int i = 1; i <= 5; i = i + 1) {
    print i * i;
}

// Every part is optional. This loop is a while loop in disguise:
int countdown = 3;
for (; countdown > 0;) {
    print countdown;
    countdown = countdown - 1;
}
print "liftoff";

// ---- break and continue -------------------------------------------------------
// continue skips to the next round, break leaves the loop.
for (int i = 1; i <= 10; i = i + 1) {
    if (i == 3) { continue; }
    if (i > 6) { break; }
    print i;
}

// ---- Nested loops: a multiplication table ----------------------------------------
for (int row = 1; row <= 3; row = row + 1) {
    for (int col = 1; col <= 4; col = col + 1) {
        print row * col;
    }
}

// ---- FizzBuzz ---------------------------------------------------------------------
for (int i = 1; i <= 15; i = i + 1) {
    bool by3 = i - i / 3 * 3 == 0;
    bool by5 = i - i / 5 * 5 == 0;

    if (by3 && by5)  { print "FizzBuzz"; }
    else if (by3)    { print "Fizz"; }
    else if (by5)    { print "Buzz"; }
    else             { print i; }
}
