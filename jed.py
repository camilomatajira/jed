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
from numbers import Number
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


def value_substitute(data: dict | list, old_regex: str, new: str) -> dict:
    compiled_regex = re.compile(old_regex, re.DOTALL)
    if isinstance(data, list):
        result = []
        for j in data:
            result.append(value_substitute(j, old_regex, new))
        return result
    elif isinstance(data, dict):
        data_copy = data.copy()
        for i in data.keys():
            data_copy[i] = value_substitute(data[i], old_regex, new)
        return data_copy
    elif isinstance(data, bool):
        data_copy = compiled_regex.sub(new, str(data))
        if re.match("^[Tt]rue$", data_copy):
            return True
        if re.match("^[Ff]alse$", data_copy):
            return False
        return data_copy
    elif isinstance(data, Number):
        data_copy = compiled_regex.sub(new, str(data))
        try:
            return float(data_copy) if "." in data_copy else int(data_copy)
        except ValueError:
            return data_copy
    elif data is None:
        data_copy = compiled_regex.sub(new, "")
        if data_copy == "":
            return None
        return data_copy
    else:
        data_copy = compiled_regex.sub(new, data)
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
