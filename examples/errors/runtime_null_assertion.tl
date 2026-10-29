// Runtime error: !! on a value that is null. Prints "before", then stops with:
//   runtime error: '!!' applied to a null value
// Use ?? to supply a default instead when null is a possible, normal outcome.

int? missing = parseInt("not a number");

print "before";
print missing!!;
print "after";
