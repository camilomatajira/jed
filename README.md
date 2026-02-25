# jed - sed for JSON

Jed is a command-line tool that brings sed's power to JSON manipulation. Unlike other JSON tools that invent new query languages from scratch, jed uses the familiar sed syntax that Unix power users already know.

If you know sed, you already know jed.

## Why jed?

Tools like `jq` are powerful but require learning a completely new language. 
Instead, I propose 'sticking to our knitting' and continuing to build upon the work of giants like `sed`, `awk`, and `vim`: Tools whose syntax has proven its worth over decades. Jed transfers that knowledge to JSON.

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

### Deleting matching sections

Use `d` to delete matching portions of JSON:

```bash
jed -e '/author/ d' file.json
```

### Substitute values

Replace text in JSON values using the familiar `s/pattern/replacement/flags` syntax:

```bash
jed -e 's/Camilo MATAJIRA/Camilo A. MATAJIRA/' file.json
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
cargo test
```

## Examples

Let's download Matthew 6, and let's search for the Lord's Prayer:
```bash
curl https://cdn.jsdelivr.net/gh/wldeh/bible-api/bibles/en-kjv/books/matthew/chapters/6.json -o mat6.json
```
Let's check it out:
```bash
cat mat6.json | jed -e 'p'
```
```json
{
  "data": [
    {
      "book": "Matthew",
      "chapter": "6",
      "text": "Take heed that ye do not your alms before men, to be seen of them: otherwise ye have no reward of your Father which is in heaven.",
      "verse": "1"
    },
    (...)
    {
      "book": "Matthew",
      "chapter": "6",
      "text": "Take therefore no thought for the morrow: for the morrow shall take thought for the things of itself. Sufficient unto the day is the evil thereof.",
      "verse": "34"
    }
  ]
}
```
Let's suppose we are only interested in verses 9 to 13 (range 8 to 12).
And we are interested interested only on the "text" and the "verse" keys (hence the regular expression "text|verse").
```bash
cat mat6.json | jed -e '/data/.8,12./text|verse/ p'
```
```json
{
  "data": [
    {
      "text": "After this manner therefore pray ye: Our Father which art in heaven, Hallowed be thy name.",
      "verse": "9"
    },
    {
      "text": "Thy kingdom come. Thy will be done in earth, as it is in heaven.",
      "verse": "10"
    },
    {
      "text": "Give us this day our daily bread.",
      "verse": "11"
    },
    {
      "text": "And forgive us our debts, as we forgive our debtors.",
      "verse": "12"
    },
    {
      "text": "And lead us not into temptation, but deliver us from evil: For thine is the kingdom, and the power, and the glory, for ever. Amen.",
      "verse": "13"
    }
  ]
}
```
Now we would like to replace the "data" key with "The Lord's prayer" (using 'S' to replace keys) and change "forgive" to "FORGIVE" (using 's' to replace values).

```bash
cat mat6.json | jed -e '/data/.8,12./text|verse/ p' | jed -e "S/data/The Lord's prayer" | jed -e "s/forgive/FORGIVE"
```
```json
{
  "The Lord's prayer": [
    {
      "text": "After this manner therefore pray ye: Our Father which art in heaven, Hallowed be thy name.",
      "verse": "9"
    },
    {
      "text": "Thy kingdom come. Thy will be done in earth, as it is in heaven.",
      "verse": "10"
    },
    {
      "text": "Give us this day our daily bread.",
      "verse": "11"
    },
    {
      "text": "And FORGIVE us our debts, as we FORGIVE our debtors.",
      "verse": "12"
    },
    {
      "text": "And lead us not into temptation, but deliver us from evil: For thine is the kingdom, and the power, and the glory, for ever. Amen.",
      "verse": "13"
    }
  ]
}
```

## TO DO's
This is just the beginning of the project. There are a lot of features that I would like to introduce.
For the near future, I would like to add the following:
* Filter commands by value (the filter applies on the values, not on the keys). Example:
```bash
jed -e ':this_value s/this_value/another_value/g' file.json
```
* Allow wildcards in the lists.
* Be able to filter not just from the beginning, but allow the key chains to start at the middle.
* Allow 'in-place' editing (like sed -i).


## Project history

Jed started as an [idea/wish](https://camilo.matajira.com/?p=638), the first prototype (v0.1) was written in Python using the Lark Parser[Python prototype](https://camilo.matajira.com/?p=638), then it was added the "substitute" command for values and keys [key and value substitution](https://camilo.matajira.com/?p=670) (v0.2), and now it was rewritten in Rust for performance, using Pest as the parser and Serde to parse JSON.
