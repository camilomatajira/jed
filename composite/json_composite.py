from abc import ABC, abstractmethod


class JSONComponent(ABC):
    @abstractmethod
    def display(self):
        pass


# Leaf classes for primitive types
class JSONLeaf(JSONComponent):
    def __init__(self, value):
        self.value = str(value)

    def display(self):
        return str(self.value)


class JSONComposite(JSONComponent):
    def __init__(self):
        self.children = []

    def add(self, element: JSONComponent):
        self.children.append(element)

    def remove(self, element: JSONComponent):
        self.children.remove(element)

    def get_child(self, index: int):
        return self.children[index]


class JSONArrayComposite(JSONComposite):
    def display(self, indent=0):
        result = ""
        if len(self.children) == 1:
            return "[" + self.children[0].display() + "]"
        if len(self.children) == 2:
            result = self.children[0].display()
            for i in range(1, len(self.children)):
                result += "," + self.children[i].display()
            return "[" + result + "]"


# Example usage:
# obj = JSONObject()
# obj.add('name', JSONString("Composite Example"))
# obj.add('isActive', JSONBoolean(True))
# arr = JSONArray()
# arr.add(JSONNumber(1))
# arr.add(JSONNumber(2))
# arr.add(JSONNull())
# obj.add('numbers', arr)

# obj.display()
