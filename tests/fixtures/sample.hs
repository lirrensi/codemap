module Sample where

data Point = Point { x :: Double, y :: Double }

newtype Age = Age Int

type Pair = (Int, Int)

class Renderable a where
    render :: a -> String

instance Renderable Point where
    render (Point x y) = "(" ++ show x ++ ", " ++ show y ++ ")"

add :: Int -> Int -> Int
add a b = a + b

greet :: String -> String
greet name = "Hello, " ++ name ++ "!"

magnitude :: Point -> Double
magnitude (Point x y) = sqrt (x * x + y * y)
