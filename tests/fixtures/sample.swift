import Foundation

struct Point {
    var x: Double
    var y: Double
}

class Vector {
    var x: Double
    var y: Double

    init(x: Double, y: Double) {
        self.x = x
        self.y = y
    }

    func magnitude() -> Double {
        return sqrt(x * x + y * y)
    }

    func scale(by factor: Double) {
        x *= factor
        y *= factor
    }
}

protocol Renderable {
    func render() -> String
}

enum Direction {
    case up
    case down
    case left
    case right
}

typealias Pair = (Int, Int)

func add(_ a: Int, _ b: Int) -> Int {
    return a + b
}

func greet(name: String) -> String {
    return "Hello, \(name)!"
}
