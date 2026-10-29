// 09 - Reading input: input(), parseInt() and ??
//
// Built-in functions:
//   input()          reads one line from standard input as a string?
//                    It returns null when there is no more input (end of file),
//                    and "" for an empty line. The line ending is removed.
//   parseInt(text)   turns text into an int?, or null if it is not a whole number
//   parseFloat(text) turns text into a float?, or null if it is not a number
//
// Both parse functions accept a string? so you can pass input() straight in.
//
// Run it and type your answers, or feed it a file:
//   tiny-compiler-vm execute 09_greeter.tb < 09_greeter.in

print "What is your name?";
string name = input() ?? "stranger";   // null (no input at all) becomes "stranger"
if (name == "") {                      // an empty line is a string, not null
    name = "stranger";
}
print "Hello, " + name + "!";

print "How old are you?";
int? age = parseInt(input());          // null if missing or not a number

if (age == null) {
    print "That is not a whole number, so I will not guess.";
} else if (age!! < 18) {
    print "You are a minor.";
    print 18 - age!!;
    print "(years until you are 18)";
} else {
    print "You are an adult.";
    print age!! + 1;
    print "(your age next year)";
}

print "What is your favourite number (decimals allowed)?";
float favourite = parseFloat(input()) ?? 0.0;
print favourite * 2.0;
print "(twice your favourite number)";
