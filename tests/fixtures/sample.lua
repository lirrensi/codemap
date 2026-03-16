local function add(a, b)
    return a + b
end

function greet(name)
    return "Hello, " .. name .. "!"
end

local M = {}

function M.multiply(a, b)
    return a * b
end

function M.divide(a, b)
    return a / b
end

return M
