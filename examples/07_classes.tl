// 07 - Classes
//
// A class bundles data (fields) and behaviour (methods).
//   * Fields are declared with an initial value, which every new object starts with.
//   * Methods are declared with `fun` and reach the object through `this`.
//   * An object is created with ClassName().
//   * `static` members belong to the class itself and are used as ClassName.member.

class Point {
    int x = 0;
    int y = 0;

    fun void moveBy(int dx; int dy) {
        this.x = this.x + dx;
        this.y = this.y + dy;
    }

    fun int distanceSquared() {
        return this.x * this.x + this.y * this.y;
    }
}

Point p = Point();
print p.x;
print p.y;

p.moveBy(3; 4);
print p.x;
print p.y;
print p.distanceSquared();

p.x = 10;                              // fields can be assigned from outside
print p.x;

// ---- Objects are shared, not copied ----------------------------------------------------
Point q = p;
q.moveBy(1; 1);
print p.x;                             // 11: p and q are the same object

Point other = Point();
print other.x;                         // 0: a new object has its own fields

// Equality of objects is identity: same object, not same contents.
print p == q;
print p == other;

// ---- A bigger example: a bank account with a static counter ------------------------------------
class Account {
    static int accountsOpened = 0;     // one copy, shared by the whole class
    static float interestRate = 0.05;

    float balance = 0.0;
    int transactions = 0;

    static fun int open() {
        Account.accountsOpened = Account.accountsOpened + 1;
        return Account.accountsOpened;
    }

    fun void deposit(float amount) {
        this.balance = this.balance + amount;
        this.transactions = this.transactions + 1;
    }

    // Returns false instead of going below zero
    fun bool withdraw(float amount) {
        if (amount > this.balance) {
            return false;
        }
        this.balance = this.balance - amount;
        this.transactions = this.transactions + 1;
        return true;
    }

    fun void addInterest() {
        this.balance = this.balance + this.balance * Account.interestRate;
    }
}

Account.open();
Account.open();
print Account.accountsOpened;          // 2

Account savings = Account();
savings.deposit(200.0);
print savings.withdraw(50.0);          // true
print savings.withdraw(500.0);         // false: not enough money
print savings.balance;
savings.addInterest();
print savings.balance;
print savings.transactions;

Account.interestRate = 0.10;           // static fields can be changed too
savings.addInterest();
print savings.balance;
