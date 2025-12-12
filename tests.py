import unittest
import json
from jed import key_substitute, value_substitute


class TestInitial(unittest.TestCase):
    def test_key_substitute_1(self):
        some_json = '{"sha": "0eb3da11ed489189963045a3d4eb21ba343736cb", "node_id": "C_kwDOAE3WVdoAKDBlYjNkYTExZWQ0ODkxODk5NjMwNDVhM2Q0ZWIyMWJhMzQzNzM2Y2I"}'
        data = json.loads(some_json)
        data = key_substitute(data, "sha", "new_sha")
        assert "new_sha" in data.keys()

    def test_key_substitute_2(self):
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

    def test_key_substitute_3(self):
        some_json = """
{
  "commit": {
    "author": {
      "name": "bigmoonbit",
      "nombre": "hola"
    }
}
}
"""
        data = json.loads(some_json)
        data = key_substitute(data, "nombre", "name")
        # Repeated keys will keep the last one
        assert data["commit"]["author"]["name"] == "hola"

    def test_key_substitute_4(self):
        some_json = """
{
  "commit": [
    { "author": "camilo" },
    { "author": "andres" }
    ]
}
"""
        data = json.loads(some_json)
        data = key_substitute(data, "author", "autor")
        # Repeated keys will keep the last one
        assert data["commit"][0]["autor"] == "camilo"
        assert data["commit"][1]["autor"] == "andres"

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

    def test_value_substitute_2(self):
        some_json = """
{
  "commit": [
    { "author": "camilo" },
    { "author": "andres" }
    ]
}
"""
        data = json.loads(some_json)
        data = value_substitute(data, "andres", "mata")
        print(data)

        assert data["commit"][1]["author"] == "mata"

    def test_value_substitute_3(self):
        some_json = """
[
    { "author": "camilo" },
    { "author": "andres" }
]
"""
        data = json.loads(some_json)
        data = value_substitute(data, "andres", "mata")
        assert data[1]
        assert data[1]["author"] == "mata"

    def test_value_substitute_4(self):
        some_json = """
{
  "commit": {
    "author": {
      "name": 5
    }
}
}
"""
        data = json.loads(some_json)
        data = value_substitute(data, "5", "6")
        assert data["commit"]["author"]["name"] == 6

    def test_value_substitute_5(self):
        some_json = """
{
  "commit": {
    "author": {
      "name": true
    }
}
}
"""
        data = json.loads(some_json)
        print("$" * 80)
        print(data)
        print("$" * 80)
        # assert False
        data = value_substitute(data, "True", "False")
        assert data["commit"]["author"]["name"] is False
