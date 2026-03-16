interface Point {
  x: number;
  y: number;
}

type Pair = [number, number];

enum Direction {
  Up,
  Down,
  Left,
  Right,
}

class Vector2D {
  constructor(public x: number, public y: number) {}

  add(other: Point): Vector2D {
    return new Vector2D(this.x + other.x, this.y + other.y);
  }

  magnitude(): number {
    return Math.sqrt(this.x * this.x + this.y * this.y);
  }
}

function distance(a: Point, b: Point): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return Math.sqrt(dx * dx + dy * dy);
}

const normalize = (v: Point): Point => {
  const len = Math.sqrt(v.x * v.x + v.y * v.y);
  return { x: v.x / len, y: v.y / len };
};
