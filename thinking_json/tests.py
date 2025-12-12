import unittest
import json
from jed_only_dicts import key_substitute, value_substitute


class TestInitial(unittest.TestCase):
    # def test_creation(self):
    #     with open("./test_no_array.json", "r") as f:
    #         data = json.load(f)
    #     print(data)
    #     assert True

    #     data = key_substitute(data, "hola
    def test_creation(self):
        some_json = '{"sha": "0eb3da11ed489189963045a3d4eb21ba343736cb", "node_id": "C_kwDOAE3WVdoAKDBlYjNkYTExZWQ0ODkxODk5NjMwNDVhM2Q0ZWIyMWJhMzQzNzM2Y2I"}'
        data = json.loads(some_json)
        data = key_substitute(data, "sha", "new_sha")
        assert "new_sha" in data.keys()

    def test_creation_2(self):
        some_json = """
{
  "commit": {
    "author": {
      "name": "bigmoonbit"
    }
}
}
"""
        data = json.loads(some_json)
        data = key_substitute(data, "a", "o")
        assert data["commit"]["outhor"]["nome"] == "bigmoonbit"

    def test_value_substitute(self):
        some_json = """
{
  "commit": {
    "author": {
      "name": "bigmoonbit"
    }
}
}
"""
        data = json.loads(some_json)
        data = value_substitute(data, "oo", "AAA")
        assert data["commit"]["author"]["name"] == "bigmAAAnbit"
