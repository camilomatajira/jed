import re


def key_substitute(data: dict, old_regex: str, new: str) -> dict:
    data_copy = data.copy()
    compiled_regex = re.compile(old_regex)
    for i in data.keys():
        if type(data[i]) is dict:
            data_copy[i] = key_substitute(data[i], old_regex, new)
            if compiled_regex.match(i):
                data_copy[compiled_regex.sub(new, i)] = data_copy[i]
                del data_copy[i]
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
        elif compiled_regex.search(data[i]):
            data_copy[i] = compiled_regex.sub(new, data[i])
    return data_copy
