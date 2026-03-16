#include <string>

struct Point {
    int x;
    int y;
};

class Vector {
public:
    Vector(int x, int y) : x_(x), y_(y) {}

    int magnitude() const {
        return x_ * x_ + y_ * y_;
    }

    void scale(int factor);

private:
    int x_;
    int y_;
};

enum class Direction {
    Up,
    Down,
    Left,
    Right
};

int add(int a, int b) {
    return a + b;
}

namespace math {
    double sqrt_approx(double n) {
        return n * 0.5;
    }
}
