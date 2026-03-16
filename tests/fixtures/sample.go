package main

type Config struct {
	Name    string
	Verbose bool
	Count   int
}

type Reader interface {
	Read(p []byte) (n int, err error)
	Close() error
}

func NewConfig(name string) *Config {
	return &Config{Name: name}
}

func (c *Config) IsVerbose() bool {
	return c.Verbose
}

func (c *Config) SetCount(n int) {
	c.Count = n
}

func add(a, b int) int {
	return a + b
}
