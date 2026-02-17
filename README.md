# jed - sed for JSON

Jed is a command-line tool that brings sed's power to JSON manipulation. Unlike other JSON tools that invent new query languages, jed uses the familiar sed syntax that Unix power users already know.

If you know sed, you already know jed.

## Why jed?

Tools like `jq` are powerful but require learning a completely new language. The Unix philosophy gave us `sed`, `awk`, and `vim` -- tools whose syntax has proven its worth over decades. Jed transfers that knowledge to JSON.

## Installation

```bash
# Build from source (requires Rust)
# Install to ~/.local/bin
make install
```

## Usage

```
jed -e '<command>' <file.json>
```
### Print matching sections

Use `p` to filter and display only matching portions of JSON:

```bash
jed -e '/author/ p' file.json
```

Specifically, 
```bash
jed -e 'p' file.json
```
Gives you back the entire JSON (identity operator).

### Substitute values

Replace text in JSON values using the familiar `s/pattern/replacement/flags` syntax:

```bash
jed -e 's/Camilo MATAJIRA/Camilo A. MATAJIRA/g' file.json
```
(flags currently not implemented, but will be in the future)

### Filter by key

Apply operations only to values under matching keys:

```bash
jed -e '/author/ s/José/Jose/g' file.json
```

### Filter by key chain

Match nested key paths using dot-separated regex patterns:

```bash
jed -e '/commit/./author/./name/ s/old/new/g' file.json
```

### Filter by array range

Operate only on specific array elements:

```bash
jed -e '1,10 s/a/X/g' file.json
```

### Filter by a mix of everything

Filter on arrays and keys all at once:
```bash
jed -e '0,1./author/./.*url/p' test.json
```


### Flags

Not ready yet.

## Type handling

Jed is JSON-aware. Substitutions intelligently handle type conversions:

- Replacing a number with a numeric string keeps it as a number
- Boolean and null values can be substituted
- Non-numeric replacements on numbers produce strings

## Development

```bash
# Run tests
cargo test --bin jed

# Debug build
cargo build
```

## TO DO's
This is just the beginning of the project. There are a lot of features that I would like to introduce.
For the near future,  I would like to add the following:
* Filter commands by value (the filter applies on the values, not on the keys). Example:
```bash
jed -e ':this_value s/this_value/another_value/g' file.json
```
* Add support to read from stdin.
* Add the command 'S' to subtitute on the keys.
* Be able to filter not just from the beginning, but allow the key chains to start at the middle.
* Write a more detailed documentation with examples.


## Project history

Jed started as an [idea/wish](https://camilo.matajira.com/?p=638), the first prototype (v0.1) was written in Python using the Lark Parser[Python prototype](https://camilo.matajira.com/?p=638), then it was added the "substitute" command for values and keys [key and value substitution](https://camilo.matajira.com/?p=670) (v0.2), and now it was rewritten in Rust for performance, using Pest as the parser and Serde to parse JSON.
