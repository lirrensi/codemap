struct Point
    x::Float64
    y::Float64
end

mutable struct Calculator
    value::Int
end

abstract type Shape end

module MathHelper
    function add(a, b)
        a + b
    end
end

function greet(name::String)
    "Hello, $name!"
end

function add(a::Int, b::Int)::Int
    a + b
end

magnitude(p::Point) = sqrt(p.x^2 + p.y^2)
