
# Questions

1. If key substitute generates two identical keys in a same object, what to do?

#################################2. 

# Pensamientos
29/12/2025
Desde un principio pense quela forma adecuada de aboardar el problema era con el design pattern de "composite".
Al final de cuentas un json es un "composite".
Sin embargo, abordar el problema asi, de una manera profesional, me parecio algo complicado.
Al final hice lo que puede con diccionarios y haciendo funcionar como pudiera.

Siempre he operado bajo el supuesto de que tengo una gasolina fija de motivacion, y que tengo que ver resultados pronto, sino voy a abandonar el proyecto.
Por eso mismo ni intente hacerlo en rust.

El tema es que tanto con los diccionarios, como con el composite no se como resolver el problema de los filtros.
Los filtros son los regex que se aplican a las keys y values.

La unica forma que se me ocurre para resolver el problema es transformando la estructure de datos con "gron".
Y aun asi, no tengo todo el problema resuelto.

Estuve buscando "useful data structures" y encontre esta: https://en.wikipedia.org/wiki/Trie
La verdad pareciera que es lo que necesito.


This is version v0.2 of jed.
Jed is a command-line tool that aims to be the spiritual successor of sed but specialized in JSON data manipulation.
I have written about this project before: https://camilo.matajira.com/?p=635 https://camilo.matajira.com/?p=638

In this realease I added the following features:
* Key substitution: You can now substitute keys in JSON objects using regex patterns.
* Value substitution: You can now substitute values in JSON objects using regex patterns.
* Output with Colors.

The speed of the tool is remarkable for a tool written in Python. It shows the powes of json.loads and the regex modules,
both written in C.

Below is the code of the project with the unit test and examples. I still haven't upload it to Github, I am waiting for v1.0.
