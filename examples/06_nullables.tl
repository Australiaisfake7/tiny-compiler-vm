// 06 - Nullable values
//
// A type followed by ? may hold null: int?, string?, Point?, ...
// A plain type such as int can never be null, and the compiler enforces that.

int? maybe = null;
int? present = 7;
int? unset;                          // a declaration without a value starts as null

print maybe;
print present;
print unset;

// ---- Checking for null ---------------------------------------------------------------
print maybe == null;
print present != null;

if (present != null) {
    print "present has a value";
}

// ---- ?? supplies a default ------------------------------------------------------------
// a ?? b gives a, or b when a is null.
print maybe ?? 100;
print present ?? 100;

string? nickname = null;
print nickname ?? "anonymous";

// ---- !! asserts that a value is not null --------------------------------------------------
// Use it to turn an int? into an int. If the value IS null the program stops with
//   runtime error: '!!' applied to a null value
int sure = present!! + 1;
print sure;
// print maybe!!;                    // uncomment to see the error

// ---- Functions that may have no answer -------------------------------------------------
fun int? safeDivide(int a; int b) {
    if (b == 0) { return null; }
    return a / b;
}

fun int? indexOf(int[4] values; int target) {
    for (int i = 0; i < 4; i = i + 1) {
        if (values[i] == target) { return i; }
    }
    return null;
}

print safeDivide(10; 2);
print safeDivide(1; 0);
print safeDivide(1; 0) ?? -1;

int[4] data = [5; 6; 7; 8];
print indexOf(data; 7);
print indexOf(data; 1);

// ---- Objects can be null too, and ?. reads a member only when the object exists ---------------
class Box {
    int value = 42;
    fun int doubled() { return this.value * 2; }
}

Box? box = null;
print box?.value;                    // null instead of an error
print box?.doubled();

box = Box();
print box?.value;
print box?.doubled();
print box!!.value;                   // !! when you are sure it exists

// ---- The built-ins parseInt and parseFloat return nullable results ---------------------------------
// They give null when the text is not a valid number.
print parseInt("123");
print parseInt("12x");
print parseInt("") ?? 0;
print parseFloat("2.5");
print parseFloat("abc") ?? 0.0;
print parseFloat("inf");             // not a finite number either, so null

// A plain value can always be stored in a nullable variable:
int? converted = 5;
print converted;
