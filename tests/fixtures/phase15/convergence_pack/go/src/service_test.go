package service

import "testing"

func TestPlanPhase63(t *testing.T) {
	if PlanPhase63() != 63 {
		t.Fatalf("expected 63")
	}
}
