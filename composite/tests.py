import unittest
from json_composite import JSONLeaf, JSONComposite, JSONArrayComposite


class TestLeaf(unittest.TestCase):
    def test_creation(self):
        leaf = JSONLeaf(1)
        assert leaf
        # Data saved us string
        assert leaf.value == "1"
        assert leaf.display() == "1"


class TestComposite(unittest.TestCase):
    def test_creation(self):
        leaf = JSONLeaf(1)
        composite = JSONArrayComposite()
        assert composite
        composite.add(leaf)
        assert composite.get_child(0) == leaf
        composite.remove(leaf)
        # Data saved us string

    def test_json_array_composite(self):
        leaf = JSONLeaf(1)
        composite = JSONArrayComposite()
        composite.add(leaf)
        assert composite.display() == "[1]"
        composite.add(JSONLeaf(2))
        assert composite.display() == "[1,2]"

    def test_json_object_composite(self):
        leaf = JSONLeaf(1)
        composite = JSONObjectComposite("name")
        composite.add(leaf)
        assert composite.display() == "[1]"
        composite.add(JSONLeaf(2))
        assert composite.display() == "[1,2]"
