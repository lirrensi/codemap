defmodule Calculator do
  def add(a, b) do
    a + b
  end

  defp private_add(a, b) do
    a + b
  end

  def multiply(a, b) do
    a * b
  end
end

defmodule Greeter do
  def greet(name) do
    "Hello, #{name}!"
  end
end
