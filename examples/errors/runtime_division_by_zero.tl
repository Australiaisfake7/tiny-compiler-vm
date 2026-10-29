// Runtime error: prints "before", then stops with:
//   runtime error: division by zero

int divisor = 0;

print "before";
print 10 / divisor;
print "after";
