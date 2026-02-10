package app

import (
	otherpkg "../other"
	utilpkg "../util"
)

func Run() string {
	_ = otherpkg.Helper()
	value := utilpkg.Helper()
	greeting := utilpkg.Greeter{}.SayHello()
	if value > 0 {
		return greeting
	}
	return "fallback"
}
