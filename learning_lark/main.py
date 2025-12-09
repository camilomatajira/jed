import re
from lark import Lark, Transformer

grammar = r"""
start: command+

command: "s/" PATTERN "/" REPLACE "/" FLAGS   -> substitute
       | "/" PATTERN "/" "d"                  -> delete
       | "/" PATTERN "/" "p"                  -> print_line

PATTERN: LETTER+
REPLACE: LETTER+
FLAGS: LETTER+

%import common.LETTER
%import common.WS
%ignore WS
"""

parser = Lark(grammar)


class SedTransformer(Transformer):
    def __init__(self, text):
        super().__init__()
        self.text = text

    def substitute(self, args):
        print(args)
        # import sys

        # sys.exit()
        pat, repl, flags = args
        count = 0 if flags and "g" in flags else 1
        self.text = re.sub(str(pat), str(repl), self.text, count=count)

    def delete(self, args):
        (pat,) = args
        self.text = "\n".join(
            line for line in self.text.splitlines() if not re.search(str(pat), line)
        )

    def print_line(self, args):
        (pat,) = args
        for line in self.text.splitlines():
            if re.search(str(pat), line):
                print(line)


script = """
s/foo/bar/g
/hello/d
"""

tree = parser.parse(script)
t = SedTransformer("foo hello\nfoo test")
t.transform(tree)
print(t.text)

# Conclusion
# This is great. The code came from chatgpt, but I had to fix a few things.
