import pkg_a.util as util_a
import pkg_b.util as util_b
from pkg_a.util import helper as helper_a
from pkg_b.util import helper as helper_b


def run_module_a():
    return util_a.helper()


def run_module_b():
    return util_b.helper()


def run_alias_a():
    return helper_a()


def run_alias_b():
    return helper_b()
