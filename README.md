# jed - sed for JSON

`jed` is a command-line tool that brings `sed`'s power to JSON manipulation. Unlike other JSON tools that invent new query languages from scratch, `jed` uses the familiar sed syntax that power users already know (and love!).

If you know `sed`, you already know `jed`.

## Why jed?

Tools like `jq` are powerful but require learning a completely new language.
Instead, I propose 'sticking to our knitting' and continuing to build upon the work of giants like `sed`, `awk`, and `vim`: Tools whose syntax has proven its worth over decades. Jed transfers that knowledge to JSON.

In this particular case, your knowledge of `sed` can transfer to `jed`, and then your investment learning `sed` 
has a higher ROI, and learning `jed` is easier.

I would recommend anyone to learn `sed` and `awk`, for this I recommend this [book](https://camilo.matajira.com/?p=354).
To learn `vim` I recommend this [book](https://camilo.matajira.com/?p=308).

## Installation

```bash
# Build from source (requires Rust)
# Install to ~/.local/bin
make install
```
## To keep in mind
* Jed traverses a JSON document recursively from top to bottom. To better understand what I mean, check serde JSON's definition
  of a JSON Value:
```
Any valid JSON data can be manipulated in the following recursive enum representation. This data structure is serde_json::Value.

enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```
* Jed defines operations (like `print`, `delete`, `value substitute`) that apply differently on the type of value (Array, Object, String, Number, Bool, Null) the program is currently on.
* Jed has operations that apply only to String, Number, Bool, Null (like `value substitute`), and others that only apply to Objects (like `key substitute`).
* There are also filters that only apply to Arrays (example: `0,10`) while others only apply to an Object key (example `/regex/`).
* There are filters that only apply to String, Number, Bool, Null like `:/regex/` (but this is still work in progress).

## Usage

```
jed -e '<command>' <file.json>
```
### Print matching sections

Use `p` to filter and display only matching portions of JSON:

```bash
jed -e '/author/ p' file.json
```
This could be read as: "If you find an Object whose key matches the regex /author/, print the key and its associated Value, if not, don't print".

One special case of the print command is the following: 
```bash
jed -e 'p' file.json
```
Gives you back the entire JSON (identity operator).

### Deleting matching sections

Use `d` to delete matching portions of JSON:

```bash
jed -e '/author/ d' file.json
```
This could be read as: "If you find an Object whose key matches the regex /author/, delete that key and its associated Value".

### Substitute values

Replace text in JSON values using the familiar `s/pattern/replacement/flags` syntax:

```bash
jed -e 's/apple/orange/' file.json
```
This could be read as: "If you find a String that matches the regex /apple/, replace that
match with 'orange'."
Substitute value also works with Number, Null and Bool.

### Substitute keys

Replace key names in JSON objects using `S/pattern/replacement`:

```bash
jed -e 'S/author/writer/' file.json
```
This could be read as: "If you find an Object key that matches the regex /author/, replace that key with 'writer'."

See the difference between `substitute values` and `substitute keys`?One operate on String, Number, Bool, Null while the other operate on the keys of an Object.

### Filter by key

Apply operations only to values under matching keys:

```bash
jed -e '/author/ s/José/Jose/g' file.json
```
This could be read as: "If you find an Object that has a key that matches the regex /author/, recursively replace /José/ with /Jose/."

### Filter by key chain

Match a sucession of Object keys using `.`

```bash
jed -e '/commit/./author/./name/ s/old/new/g' file.json
```
This could be read as: "Wherever you find a succession of objects whose keys match the following three patterns /commit/ /author/ /name/, only there replace /old/ with /new/."

Something like this would match, and "old" would be replaced by "new":
```
{
  "commit": {
    "author": {
      "name": "old",
    }
}
```
Result:
```
{
  "commit": {
    "author": {
      "name": "new",
    }
}
```


### Filter by array range

Operate only on specific array elements:

```bash
jed -e '0,1 s/a/X/g' file.json
```
This could be read as: "Wherever you find an array, only on its elements 0 to 1 (and descend recursively) replace /a/ with /X/."

The output of applying that command here:
```
{
  "commit": [
    { "author": "camilo" },
    { "author": "andres" }
    { "author": "camilo andres" }
    ]
}
```
Would be
```
{
  "commit": [
    { "author": "cXmilo" },
    { "author": "Xndres" }
    { "author": "camilo andres" }
    ]
}
```

### Filter by a mix of everything

Filter on arrays and keys all at once:
```bash
jed -e '0,1./author/./.*url/p' test.json
```
This could be read as: "Wherever you find a succession of an array (elements 0 and 1), followed by two Objects, the first Object key matches /author/ and the second Object's key matches /.*url/, then print."

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
And we are only interested in the "text" and the "verse" keys (hence the regular expression "text|verse").
```bash
# Filtering from the root 'data'
cat mat6.json | jed -e '/data/.8,12./text|verse/ p'
# or
# Filters can start at any depth
cat mat6.json | jed -e '8,12./text|verse/ p'
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
If you know `sed`, you will notice that I have just implemented the most basic commands.
For the near future, I would like to add the following:
* Filter commands by value (the filter applies to the values, not to the keys). This is work in progress. Example:
```bash
jed -e ':this_value s/this_value/another_value/g' file.json
```
* Allow 'in-place' editing (like sed -i).
* Support reading from multiple files.
* Remove the need for '-e' to pass an expression.
* And more!


## Project history

Jed started as an [idea/wish](https://camilo.matajira.com/?p=638), the first prototype (v0.1) was written in Python using the Lark parser, then the "substitute" command for values and keys was added ([key and value substitution](https://camilo.matajira.com/?p=670)) (v0.2), and now it was rewritten in Rust for performance, using Pest as the parser and Serde to parse JSON.
