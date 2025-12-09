from abc import ABC, abstractmethod


class JSONComponent(ABC):
    pass


# Leaf classes for primitive types
class JSONLeaf(JSONComponent):
    def __init__(self, value):
        self.value = value

    # def display(self, indent=0):
    #     print(" " * indent + f'"{self.value}"')


# class JSONComposite(JSONElement):
#     def __init__(self):
#         self.elements = []

#     def add(self, element: JSONElement):
#         self.elements.append(element)

#     def display(self, indent=0):
#         print(" " * indent + "[")
#         for i, e in enumerate(self.elements):
#             e.display(indent + 2)
#             if i < len(self.elements) - 1:
#                 print(",")
#         print(" " * indent + "]")


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
