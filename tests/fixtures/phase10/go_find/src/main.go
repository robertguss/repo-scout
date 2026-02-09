package main

type Greeter struct{}

func (g Greeter) SayHello() string {
	return "hello"
}

func HelperToken() int {
	return 1
}
