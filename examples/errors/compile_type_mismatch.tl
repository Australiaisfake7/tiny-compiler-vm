// Compile error: a value of the wrong type.
// Nothing runs at all; the compiler stops with:
//   semantic error: 'age' is declared as 'int', but assigned a value of type 'string'

print "this line never runs";
int age = "twelve";
