import unittest
from json_composite import JSONLeaf


class TestLeaf(unittest.TestCase):
    def test_creation(self):
        leaf = JSONLeaf(1)
        assert leaf
        # assert False
        # assert leaf
