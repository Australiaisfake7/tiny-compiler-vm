# tl examples

Twelve small programs that tour the language, plus a folder of programs that fail on
purpose so you can see what the error messages look like.

## Build the toolchain

From the project folder (the one with `Cargo.toml` and `src/`), using Rust 1.85 or newer:

```
cargo build --release
```

The binary is `target/release/tiny-compiler-vm`. These examples need the **fixed build**
from `src_fixed.zip`; against the original sources, three of them behave differently:

- `09_greeter.tl` and `10_statistics.tl` pass a `string?` to `parseInt` / `parseFloat`,
  which the original compiler rejects, and they rely on `input()` returning `""` for an
  empty line and `null` only at the end of the input.
- `06_nullables.tl` prints `null` for `parseFloat("inf")`, where the original gives `inf`.

## Compile and run one program

A program is a `.tl` file. Compiling writes a bytecode file next to it, and executing
runs that file:

```
tiny-compiler-vm compile 01_hello.tl         # writes 01_hello.tb
tiny-compiler-vm execute 01_hello.tb
```

Programs that read input take it from standard input, so you can type it or redirect a file:

```
tiny-compiler-vm compile 09_greeter.tl
tiny-compiler-vm execute 09_greeter.tb < 09_greeter.in
```

## Run everything

Put this folder inside the project (next to `target/`), or give the script the binary's path:

```
./run_examples.sh                              # print the output of every program
./run_examples.sh path/to/tiny-compiler-vm     # same, with an explicit binary
./run_examples.sh --check                      # compare with the files in expected/
```

`--check` prints PASS or FAIL per program and exits with a non-zero status if any differ,
which makes it a handy regression test while you change the compiler. Use `--update` to
rewrite `expected/` after an intentional change.

## The programs

| File | Shows |
|---|---|
| `01_hello.tl` | `print`, comments, literals, string joining, multi-line strings |
| `02_types_and_operators.tl` | `int`, `float`, `string`, `bool`; arithmetic, comparison and logic; short-circuiting |
| `03_control_flow.tl` | `if` / `else if` / `else`, `while`, `for`, `break`, `continue`, FizzBuzz |
| `04_functions.tl` | `fun`, parameters, return types, `void`, globals, recursion |
| `05_arrays.tl` | array literals and fills, indexing, nested arrays, sharing, equality, passing to functions |
| `06_nullables.tl` | `T?`, `null`, `??`, `!!`, `?.`, functions that may return null, `parseInt` / `parseFloat` |
| `07_classes.tl` | fields, methods, `this`, `static` members, objects as references |
| `08_inheritance.tl` | `class Child : Parent`, `override`, polymorphism over an array |
| `09_greeter.tl` | reading input with `input()`, `parseInt`, `parseFloat`; uses `09_greeter.in` |
| `10_statistics.tl` | a read loop that ends on an empty line or end of input; uses `10_statistics.in` |
| `11_data_structures.tl` | a stack, a queue and a reversible linked list built from nullable fields |
| `12_algorithms.tl` | sieve of Eratosthenes, bubble sort, binary search, Collatz, Fibonacci |
| `errors/compile_*.tl` | five compile-time errors: type mismatch, missing return, syntax, undeclared name, wrong argument count |
| `errors/runtime_*.tl` | four runtime errors: index out of bounds, division by zero, `!!` on null, integer overflow |

Each error program starts with a comment that states the message you should see.

## Language cheat sheet

```
// comment                              print expression;
int a = 1;  float f = 1.5;  string s = "text";  bool b = true;
int? maybe = null;                      // T? may hold null; a declaration with no value is null

if (c) { ... } else if (d) { ... } else { ... }
while (c) { ... }          for (int i = 0; i < n; i = i + 1) { ... }          break;  continue;

fun int add(int a; int b) { return a + b; }       // parameters and arguments use ';'
fun void hello() { print "hi"; }

int[3] xs = [1; 2; 3];     int[5] zeros = [0: 5];     xs[0] = 9;     // fixed length
a ?? b          // a, or b when a is null
x!!             // x, or a runtime error when x is null
obj?.member     // the member, or null when obj is null

class Animal { string name = "x"; fun string sound() { return "..."; } }
class Dog : Animal { fun override string sound() { return "woof"; } }
Animal a = Dog();     a.sound();     // static members: static int n = 0;  Animal.n
```

Built-ins: `input()` returns `string?`, `parseInt(string?)` returns `int?`,
`parseFloat(string?)` returns `float?`.

## Things that surprise newcomers

- Separators are semicolons everywhere: parameters, call arguments and array elements.
- There is no `%`, `+=`, `++` or ternary operator. Use `n - n / d * d` for the remainder
  and `i = i + 1` to count.
- There is no implicit conversion. `1 + 2.0` and `float f = 1;` are errors, and `+` joins
  a string only with another string, so print a label and its number on separate lines.
- `int / int` truncates toward zero, and integer overflow stops the program.
- Methods are declared with `fun` and reach their fields through `this.`.
  Objects are created with `ClassName()` and cannot take constructor arguments; set the
  fields afterwards or give them initial values in the class body.
- Arrays and objects are shared when assigned, not copied. Array equality compares
  contents, object equality compares identity.
- `int[2][3]` is read from the inside out: 3 rows of 2 elements, indexed `[row][column]`.
- Functions and classes are declared at the top level, and a function can only call
  functions declared above it (or itself).
- A non-nullable variable must be given a value. Only `T?` variables can be declared bare.
- Strings have no escape sequences; a string runs to the next `"` and may contain newlines.
