# Rcclex Regex Engine

## Syntax

Generally, [Posix ERE](https://en.wikibooks.org/wiki/Regular_Expressions/POSIX-Extended_Regular_Expressions) syntax is adopted.

features and limitations:
- `^`, `$` are not supported (as we always perform full matches)
- non-greedy operators (`*?`, `+?`, `??`) are not supported (as we only perform full matches)
- lookaheads (`?<=`, `?>=`, `?=`, `?!=`) are not supported
- word boundaties (`\b`) are not supported
- character classes use not POSIX syntax, but [perl syntax](https://en.wikipedia.org/wiki/Regular_expression#Character_classes)
- any characted can be matched by its ascii id

### Metacharacters

| metachar  | usage                                        |
|-----------|----------------------------------------------|
| `(...)`   | define subexpr                               |
| `\A...\Z` | define subexpr and capture group             |
| `{n}`     | match expr, repeated `n` times               |
| `{n,m}`   | match expr, repeated from `n` to `m` times   |
| `{n,}`    | match expr, repeated `n` or more times       |
| `{,m}`    | match expr, repeated `m` or less times       |
| `*`       | same as `{0,}`                               |
| `+`       | same as `{1,}`                               |
| `?`       | same as `{0,1}`                              |
| `|`       | match either left-size or right-side expr    |
| `[...]`   | match charset                                |
| `[^...]`  | match complemented charset                   |
| `.`       | match any char                               |
| `\xnn`    | match char with id define by hex number `nn` |

### Charset
- All defined escaped-characters
- Additional escape characters: `\-` and `\]`
- `a-b` matches any char within range `[a, b]`

## Implementation

- `charset.rs` &mdash; implement charset as a bitmap for all ASCII characters (only used by Lexer and Parser)
- `lexer.rs` &mdash; implement lexer
- `build_nfa.rs` &mdash; convert lexer output into 1-nfa ([thompson algorithm](https://en.wikipedia.org/wiki/Thompson%27s_construction) + [resolve epsilon closures](https://www.geeksforgeeks.org/conversion-of-epsilon-nfa-to-nfa/))
- `build_dfa.rs` &mdash; [determinize 1-nfa](https://dsacl3-2020.github.io/slides/fsa-determinization.pdf).
- `compile.rs` &mdash; provide interface for the compilation pipeline
