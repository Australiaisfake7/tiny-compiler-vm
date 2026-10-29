// 04 - Functions
//
//   fun <return type> name(<type> parameter; <type> parameter) { ... }
//
// Parameters are separated by semicolons, just like call arguments.
// Functions live at the top level, can use global variables declared above them,
// and can call themselves or any function declared above them.

int calls = 0;                       // a global variable

fun int square(int x) {
    return x * x;
}

fun float average(float a; float b) {
    return (a + b) / 2.0;
}

// A function with no result is declared void.
fun void greet(string name; int times) {
    for (int i = 0; i < times; i = i + 1) {
        print "hello, " + name;
    }
}

// Functions can change globals
fun void countCall() {
    calls = calls + 1;
}

// ---- Recursion ----------------------------------------------------------------
fun int factorial(int n) {
    if (n <= 1) { return 1; }
    return n * factorial(n - 1);
}

fun int fibonacci(int n) {
    if (n < 2) { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fun int gcd(int a; int b) {
    if (b == 0) { return a; }
    return gcd(b; a - a / b * b);
}

fun int power(int base; int exponent) {
    if (exponent == 0) { return 1; }
    return base * power(base; exponent - 1);
}

// Every path of a non-void function must return a value; otherwise the compiler
// reports: function 'name' is missing a return statement on some code path.
fun string sign(int n) {
    if (n < 0) { return "negative"; }
    if (n == 0) { return "zero"; }
    return "positive";
}

// ---- Calling them ---------------------------------------------------------------
print square(12);
print average(3.0; 4.0);
greet("tl"; 2);

countCall();
countCall();
countCall();
print calls;

print factorial(10);
print fibonacci(15);
print gcd(48; 18);
print power(2; 20);

print sign(-5);
print sign(0);
print sign(8);

// Calls are expressions, so they nest
print square(square(3));
print power(square(2); 3);
