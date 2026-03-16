package com.example;

public class Calculator {
    private int value;

    public Calculator(int initial) {
        this.value = initial;
    }

    public int add(int x) {
        this.value += x;
        return this.value;
    }

    private void reset() {
        this.value = 0;
    }
}

interface Drawable {
    void draw();
    String getName();
}

enum Status {
    ACTIVE,
    INACTIVE,
    PENDING
}

class Helper {
    static int multiply(int a, int b) {
        return a * b;
    }
}
