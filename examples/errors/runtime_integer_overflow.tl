// Runtime error: integer arithmetic is checked and never wraps around.
//   runtime error: integer overflow in '9223372036854775807 + 1'

int biggest = 9223372036854775807;

print "before";
print biggest + 1;
print "after";
