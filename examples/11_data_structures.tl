// 11 - Data structures: a stack, a queue and a linked list
//
// Classes that refer to other objects, nullable fields that mark "nothing here",
// and the ?? / !! / ?. operators working together.

// One link in a chain of values. `next` is null at the end of the chain.
class Node {
    int value = 0;
    Node? next = null;
}

// ---- Stack: last in, first out ---------------------------------------------------------
class Stack {
    Node? top = null;
    int size = 0;

    fun void push(int value) {
        Node node = Node();
        node.value = value;
        node.next = this.top;
        this.top = node;
        this.size = this.size + 1;
    }

    fun int? pop() {
        if (this.top == null) { return null; }
        int value = this.top!!.value;
        this.top = this.top!!.next;
        this.size = this.size - 1;
        return value;
    }

    fun int? peek() {
        return this.top?.value;
    }
}

Stack stack = Stack();
stack.push(10);
stack.push(20);
stack.push(30);
print stack.size;
print stack.peek();
print stack.pop();
print stack.pop();
print stack.pop();
print stack.pop();                      // empty now: null
print stack.peek();                     // also null

// ---- Queue: first in, first out ------------------------------------------------------------
class Queue {
    Node? head = null;
    Node? tail = null;
    int size = 0;

    fun void enqueue(int value) {
        Node node = Node();
        node.value = value;

        if (this.tail == null) {
            this.head = node;
            this.tail = node;
        } else {
            this.tail!!.next = node;
            this.tail = node;
        }
        this.size = this.size + 1;
    }

    fun int? dequeue() {
        if (this.head == null) { return null; }
        int value = this.head!!.value;
        this.head = this.head!!.next;
        if (this.head == null) { this.tail = null; }
        this.size = this.size - 1;
        return value;
    }
}

Queue queue = Queue();
queue.enqueue(1);
queue.enqueue(2);
queue.enqueue(3);
print queue.dequeue();
print queue.dequeue();
queue.enqueue(4);
print queue.dequeue();
print queue.dequeue();
print queue.dequeue() ?? -1;           // empty: use a default

// ---- A linked list built by hand and reversed -------------------------------------------------
fun Node? build(int count) {
    Node? head = null;
    for (int i = count; i >= 1; i = i - 1) {
        Node node = Node();
        node.value = i;
        node.next = head;
        head = node;
    }
    return head;
}

fun void printList(Node? start) {
    Node? current = start;
    while (current != null) {
        print current!!.value;
        current = current!!.next;
    }
}

fun Node? reverse(Node? start) {
    Node? previous = null;
    Node? current = start;
    while (current != null) {
        Node? following = current!!.next;
        current!!.next = previous;
        previous = current;
        current = following;
    }
    return previous;
}

Node? list = build(4);
print "forward:";
printList(list);
list = reverse(list);
print "reversed:";
printList(list);
