class Point {
  final double x;
  final double y;

  Point(this.x, this.y);

  double magnitude() {
    return (x * x + y * y);
  }
}

mixin Logger {
  void log(String msg) {
    print(msg);
  }
}

enum Direction { up, down, left, right }

typedef Pair = (int, int);

int add(int a, int b) {
  return a + b;
}

String greet(String name) {
  return 'Hello, $name!';
}
