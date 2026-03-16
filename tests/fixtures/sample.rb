module MathHelper
  def self.add(a, b)
    a + b
  end

  def self.multiply(a, b)
    a * b
  end
end

class Calculator
  attr_reader :value

  def initialize(initial = 0)
    @value = initial
  end

  def add(x)
    @value += x
    @value
  end

  def reset!
    @value = 0
  end

  private

  def secret
    "hidden"
  end
end

class << Calculator
  def version
    "1.0"
  end
end
