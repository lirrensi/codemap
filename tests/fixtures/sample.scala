package com.example

case class Point(x: Int, y: Int)

trait Renderable {
  def render(): String
}

class Calculator(private var value: Int = 0) {
  def add(x: Int): Int = {
    value += x
    value
  }

  private def reset(): Unit = {
    value = 0
  }
}

object Logger {
  def log(msg: String): Unit = {
    println(msg)
  }
}

enum Color {
  case Red, Green, Blue
}

def greet(name: String): String = {
  s"Hello, $name!"
}

def add(a: Int, b: Int): Int = a + b
