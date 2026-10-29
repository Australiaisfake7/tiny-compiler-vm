// Compile error: a function that does not return on every path.
//   semantic error: function 'classify' is missing a return statement on some code path

fun string classify(int n) {
    if (n > 0) {
        return "positive";
    }
}

print classify(1);
