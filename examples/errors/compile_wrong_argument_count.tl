// Compile error: calling a function with the wrong number of arguments.
//   semantic error: 'add' expects 2 parameters, but 1 was given

fun int add(int a; int b) {
    return a + b;
}

print add(1);
