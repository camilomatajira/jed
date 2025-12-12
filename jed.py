#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# dependencies = ["lark>=1.3.1", "pygments>=2.15.1"]
# ///


import argparse
import sys
import re
import json
from lark import Lark, Transformer
from pygments import highlight
from pygments.lexers import JsonLexer
from pygments.formatters import Terminal256Formatter


grammar = r"""
start: command+

command: JREGEX* JVALUE? "s/" OLD_PATTERN "/" NEW_PATTERN "/" FLAGS   -> jed_substitute_value_regex
       | JREGEX* JVALUE? "S/" OLD_PATTERN "/" NEW_PATTERN "/" FLAGS   -> jed_substitute_key_regex

REGEX: /[a-zA-Z0-9 \[\]+.?*_^-]+/
JREGEX: "/"REGEX"/""."?
JVALUE: ":/"REGEX"/"
NEW_PATTERN: REGEX
OLD_PATTERN: REGEX
FLAGS: LETTER+

%import common.LETTER
%import common.WS
%ignore WS
"""


class SedTransformer(Transformer):
    def __init__(self, text):
        super().__init__()
        self.text = text

    def jed_substitute_value_regex(self, args):
        regexp, replace, flags = args
        self.text = value_substitute(self.text, regexp, replace)

    def jed_substitute_key_regex(self, args):
        regexp, replace, flags = args
        self.text = key_substitute(self.text, regexp, replace)


def key_substitute(data: dict, old_regex: str, new: str) -> dict:
    data_copy = data.copy()
    compiled_regex = re.compile(old_regex)
    for i in data.keys():
        if type(data[i]) is dict:
            data_copy[i] = key_substitute(data[i], old_regex, new)
            if compiled_regex.match(i):
                data_copy[compiled_regex.sub(new, i)] = data_copy[i]
                del data_copy[i]
        elif type(data[i]) is list:
            result = []
            for j in data[i]:
                result.append(key_substitute(j, old_regex, new))
            data_copy[i] = result
        elif compiled_regex.search(i):
            data_copy[compiled_regex.sub(new, i)] = data[i]
            del data_copy[i]
    return data_copy


def value_substitute(data: dict, old_regex: str, new: str) -> dict:
    data_copy = data.copy()
    compiled_regex = re.compile(old_regex)
    for i in data.keys():
        if type(data[i]) is dict:
            data_copy[i] = value_substitute(data[i], old_regex, new)
        elif type(data[i]) is list:
            result = []
            for j in data[i]:
                result.append(value_substitute(j, old_regex, new))
            data_copy[i] = result
        elif compiled_regex.search(str(data[i])):
            data_copy[i] = compiled_regex.sub(new, str(data[i]))
    return data_copy


def pretty_print_dictionary(data: dict):
    json_data = json.dumps(data, indent=4)
    print(highlight(json_data, JsonLexer(), Terminal256Formatter(style="dracula")))


if __name__ == "__main__":
    argument_parser = argparse.ArgumentParser(
        prog="jed",
        description="Sed for json!",
    )
    argument_parser.add_argument("jed_script")
    args = argument_parser.parse_args()

    grammar_parser = Lark(grammar)
    tree = grammar_parser.parse(args.jed_script)
    data = json.loads(sys.stdin.read())
    t = SedTransformer(data)
    t.transform(tree)
    pretty_print_dictionary(t.text)


# vim: set syntax=python filetype=python:
