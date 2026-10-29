// Runtime error: the program compiles, prints "before", then stops with:
//   runtime error: index 5 is out of bounds for an array of length 3

int[3] items = [10; 20; 30];
int position = 5;

print "before";
print items[position];
print "after";
