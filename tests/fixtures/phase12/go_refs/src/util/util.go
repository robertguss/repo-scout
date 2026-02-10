package util

type Greeter struct{}

type Speaker interface {
	SayHello() string
}

type SpeakerAlias = Speaker

func Helper() int {
	return HelperToken()
}

func HelperToken() int {
	return 1
}

func (g Greeter) SayHello() string {
	return helper()
}

func helper() string {
	return "hello"
}
