import unittest
import json
from jed import key_substitute, value_substitute


class TestInitial(unittest.TestCase):
    def test_key_substitute_1(self):
        some_json = '{"sha": "0eb3da11ed489189963045a3d4eb21ba343736cb", "node_id": "C_kwDOAE3WVdoAKDBlYjNkYTExZWQ0ODkxODk5NjMwNDVhM2Q0ZWIyMWJhMzQzNzM2Y2I"}'
        data = json.loads(some_json)
        data = key_substitute(data, "sha", "new_sha")
        assert "new_sha" in data.keys()

    def test_key_substitute_recursivity(self):
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

    def test_key_substitute_repeated_keys_keeps_last(self):
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

    def test_key_substitute_recursivity_inside_lists(self):
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

    def test_value_substitute_recursivity_inside_lists(self):
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
        assert data["commit"][1]["author"] == "mata"

    def test_value_substitute_recursivity_with_list_in_the_root(self):
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

    def test_value_substitute_numbers_can_be_replaced(self):
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

    def test_value_substitute_booleans_can_be_modified(self):
        some_json = """
{
  "commit": {
    "author": {
      "name": true
    }
}
}
"""
        # TODO
        # Problem, true is interpreted as True, and I create the text "True"
        # So it's hard for the user to know how to replace booleans
        data = json.loads(some_json)
        data = value_substitute(data, "True", "False")
        assert data["commit"]["author"]["name"] is False

    def test_value_substitute_random_bug(self):
        some_json = """ 
{
"sha": "03cb1e19da91f0df728914d4c8717f7490df04e4"
}
"""
        data = json.loads(some_json)
        data = value_substitute(data, ".+", "hola")
        assert data["sha"] == "hola"

    def test_value_substitute_numbers_can_be_replaced_2(self):
        some_json = """ 
{
"sha": 0
}
"""
        data = json.loads(some_json)
        data = value_substitute(data, ".+", "hola")
        assert data["sha"] == "hola"

    def test_value_substitute_nulls_can_be_replaced(self):
        some_json = """ 
{
"sha": null
}
"""
        data = json.loads(some_json)
        data = value_substitute(data, ".*", "hola")
        assert data["sha"] == "hola"

    def test_value_substitute_new_lines_are_replaced(self):
        some_json = """ 
{
"sha": "a\\nb"
}
"""
        data = json.loads(some_json)
        data = value_substitute(data, ".+", "hola")
        print(data)
        assert data["sha"] == "hola"
