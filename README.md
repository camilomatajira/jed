
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
