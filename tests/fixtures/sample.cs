using System;

public class Calculator
{
    private int _value;

    public Calculator(int initial)
    {
        _value = initial;
    }

    public int Add(int x)
    {
        _value += x;
        return _value;
    }

    private void Reset()
    {
        _value = 0;
    }
}

public interface IDrawable
{
    void Draw();
    string GetName();
}

public enum Status
{
    Active,
    Inactive,
    Pending
}

public struct Point
{
    public int X;
    public int Y;
}

public record User(string Name, int Age);
