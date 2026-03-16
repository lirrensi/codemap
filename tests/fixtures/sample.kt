package com.example

data class Point(val x: Int, val y: Int)

enum class Direction {
    UP, DOWN, LEFT, RIGHT
}

interface Renderable {
    fun render(): String
}

class Calculator(private var value: Int = 0) {
    fun add(x: Int): Int {
        value += x
        return value
    }

    private fun reset() {
        value = 0
    }
}

object Logger {
    fun log(msg: String) {
        println(msg)
    }
}

typealias Pair = Pair<Int, Int>

fun greet(name: String): String {
    return "Hello, $name!"
}

fun add(a: Int, b: Int): Int {
    return a + b
}
