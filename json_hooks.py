import json
import requests


def custom_hook(dct):
    print("*" * 80)
    print(dct)
    print("*" * 80)
    return "DONE"
    # Maybe transform dict to a custom object
    # return MyClass(**dct)


data = requests.get("https://api.github.com/repos/jqlang/jq/commits?per_page=5").content
# result = json.loads(data, object_hook=custom_hook)
# print(result)

# Conclusion
# object_hook va tratando cada subjson de los mas profundos a los que estan mas cerca de la raiz.
# y Uno puede ir transformando el contenido.
result = json.loads(data, object_pairs_hook=custom_hook)
print(result)
