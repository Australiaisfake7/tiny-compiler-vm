// 08 - Inheritance and polymorphism
//
//   class Child : Parent { ... }
//
// A child class gets all of its parent's fields and methods. To replace a parent's
// method write `fun override` with the same name, parameters and return type.
// A variable of the parent type can hold any child, and calling a method on it runs
// the child's version.

class Shape {
    string label = "shape";

    fun float area() {
        return 0.0;
    }

    fun string describe() {
        return this.label;
    }
}

class Rectangle : Shape {
    float width = 2.0;
    float height = 3.0;

    fun override float area() {
        return this.width * this.height;
    }
}

class Circle : Shape {
    float radius = 1.0;

    fun override float area() {
        return 3.14159 * this.radius * this.radius;
    }

    fun override string describe() {
        return "circle (" + this.label + ")";
    }
}

// A grandchild: Square is a Rectangle is a Shape
class Square : Rectangle {
    fun void setSide(float side) {
        this.width = side;
        this.height = side;
    }
}

Rectangle r = Rectangle();
r.label = "rectangle";

Circle c = Circle();
c.radius = 2.0;

Square s = Square();
s.label = "square";
s.setSide(4.0);

// One array that holds all kinds of shapes
Shape[3] shapes = [r; c; s];

for (int i = 0; i < 3; i = i + 1) {
    print shapes[i].describe();        // chooses the right describe() at run time
    print shapes[i].area();
}

// A plain Shape works as well
Shape generic = Shape();
print generic.describe();
print generic.area();

// Functions can take the parent type and accept any child
fun float totalArea(Shape[3] all) {
    float total = 0.0;
    for (int i = 0; i < 3; i = i + 1) {
        total = total + all[i].area();
    }
    return total;
}

print totalArea(shapes);
