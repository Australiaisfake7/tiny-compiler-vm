// 02 - Types and operators
//
// The basic types are int (64 bit), float, string and bool.
// There is no implicit conversion: 1 and 1.0 are different types.

int count = 10;
float ratio = 2.5;
string label = "total";
bool ready = true;

print label + ": " + "ok";
print count;
print ratio;
print ready;

// ---- Arithmetic -----------------------------------------------------------
print 2 + 3 * 4;          // * and / bind tighter than + and -
print (2 + 3) * 4;        // parentheses override the order
print 10 - 2 - 3;         // left to right: (10 - 2) - 3
print -count;             // unary minus

// Integer division truncates toward zero
print 7 / 2;
print -7 / 2;

// Floats keep their fraction
print 7.0 / 2.0;
print 1.0 / 3.0;

// There is no % operator, but the remainder is easy to compute
int remainder = 17 - 17 / 5 * 5;
print remainder;

// ---- Comparison -----------------------------------------------------------
print 3 < 5;
print 3 >= 5;
print 4 == 4;
print 4 != 4;
print 1.5 < 2.5;
print "apple" == "apple";
print "apple" == "pear";

// ---- Logic ----------------------------------------------------------------
print true && false;
print true || false;
print !true;
print 1 < 2 && 2 < 3;
print 1 > 2 || 2 > 1;

// && and || stop as soon as the answer is known, so the right side is skipped here
int zero = 0;
print zero != 0 && 10 / zero > 1;

// ---- Mixing types is a compile error ----------------------------------------
// print 1 + 2.0;          // cannot apply '+' to operands of type 'int' and 'float'
// print "n = " + 5;       // cannot apply '+' to operands of type 'string' and 'int'
// float f = 1;            // 'f' is declared as 'float', but assigned a value of type 'int'
// Write 1.0 instead of 1 wherever a float is expected. Strings can only be joined
// with other strings, so print a label and its number on separate lines.

// ---- Overflow is detected -----------------------------------------------------
// Integer arithmetic never wraps around silently. Uncommenting the next two
// lines makes the program stop with: integer overflow in '9223372036854775807 + 1'
// int big = 9223372036854775807;
// print big + 1;
