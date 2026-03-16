#include <stdio.h>

struct Point {
    int x;
    int y;
};

enum Color {
    RED,
    GREEN,
    BLUE
};

typedef struct {
    int width;
    int height;
} Rectangle;

int add(int a, int b) {
    return a + b;
}

void print_point(struct Point p) {
    printf("(%d, %d)\n", p.x, p.y);
}

static int helper(int x) {
    return x * 2;
}
