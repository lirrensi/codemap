type point = { x : float; y : float }

type direction = Up | Down | Left | Right

module Calculator = struct
  let add a b = a + b

  let multiply a b = a * b
end

let greet name =
  "Hello, " ^ name ^ "!"

let magnitude p =
  sqrt (p.x *. p.x +. p.y *. p.y)

let add a b = a + b
