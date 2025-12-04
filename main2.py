import re
from lark import Lark, Transformer

grammar = r"""
start: command+

command: "s/" PATTERN "/" REPLACE "/" FLAGS   -> substitute
       | "/" REGEX "/" "," "/" REGEX "/" "s/" PATTERN "/" REPLACE "/" FLAGS   -> substitute_regex
       | JREGEX+ "Z/" PATTERN "/" REPLACE "/" FLAGS   -> jed_substitute_regex
       | "/" PATTERN "/" "d"                  -> delete
       | "/" PATTERN "/" "p"                  -> print_line
       | "a" PATTERN                          -> append_line
       | INT","INT "A" PATTERN                  -> append_line_range
       | "i" PATTERN                          -> insert_line

PATTERN: /[a-zA-Z0-9 ]+/
REGEX: /[a-zA-Z0-9 \[\]+.?]+/
JREGEX: "/"/[a-zA-Z0-9 \[\]+.?]+/"/""."?
REPLACE: LETTER+
FLAGS: LETTER+

%import common.LETTER
%import common.INT
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

    def append_line(self, args):
        (pat,) = args
        result = ""
        for line in self.text.splitlines():
            result += line + "\n" + pat + "\n"
        self.text = result

    def append_line_range(self, args):
        print(args)
        (start, end, pat) = args
        result = ""
        for i, line in enumerate(self.text.splitlines()):
            if i >= int(start) and i <= int(end):
                result += line + "\n" + pat + "\n"
            else:
                result += line + "\n"
        self.text = result

    def insert_line(self, args):
        (pat,) = args
        result = ""
        for i, line in self.text.splitlines():
            result += pat + "\n" + line + "\n"
        self.text = result

    def substitute_regex(self, args):
        # (pat,) = args
        print(args)
        import sys

        sys.exit()

    def jed_substitute_regex(self, args):
        # (pat,) = args
        print(args)
        import sys

        sys.exit()


script = """
s/foo/bar/g
/hello/d
aCamilo
"""
script = """
aCamilo
"""
script = """
iHola
"""
script = """
1,2 AHola Camilo Andres
"""
script = """
/foo/,/Andres/ s/o/a/g
"""
script = """
/foo/./Andres/./bar/ Z/o/a/g
"""

tree = parser.parse(script)
t = SedTransformer("foo hello\nfoo test\nAndres")
t.transform(tree)
print(t.text)

# Conclusion
# This is great. The code came from chatgpt, but I had to fix a few things.
