class Calculator:
    """A simple calculator."""

    def __init__(self, initial=0):
        self.value = initial

    def add(self, x):
        self.value += x
        return self.value

    @staticmethod
    def version():
        return "1.0"


class Vector:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def magnitude(self):
        return (self.x**2 + self.y**2) ** 0.5


def greet(name):
    return f"Hello, {name}!"


def _private_helper(x, y):
    return x + y
