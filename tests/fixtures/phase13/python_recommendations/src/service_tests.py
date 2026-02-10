from src.service import compute_plan


def test_compute_plan_suffix():
    assert compute_plan() == 1
