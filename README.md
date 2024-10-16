# RCC Lex

Lexer generator

- build: `zig build`
- usage: `rcclex [< config.yaml] [> output]`

## ymlz limitations:

- characters outside token list may only be escaped with `'\o'`, where o is their octal representation (i.e. `\40` for `' '`, etc.)
- `' '` character within string should always be replaced with `'\40'`
- `'#'` character within string should always be replaced with `'\43'`
- `'>'` character within string should always be replaced with `'\76'`
- `'|'` character within string should always be replaced with `'\174'`

## TODO

- better yaml parser
- tests
- add dfa minimization
