<?php

class Calculator
{
    private int $value;

    public function __construct(int $initial = 0)
    {
        $this->value = $initial;
    }

    public function add(int $x): int
    {
        $this->value += $x;
        return $this->value;
    }

    private function reset(): void
    {
        $this->value = 0;
    }
}

interface Drawable
{
    public function draw(): void;
    public function getName(): string;
}

trait Loggable
{
    public function log(string $msg): void
    {
        echo $msg;
    }
}

enum Status: string
{
    case Active = 'active';
    case Inactive = 'inactive';
}

function greet(string $name): string
{
    return "Hello, $name!";
}
