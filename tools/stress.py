#!/usr/bin/env python3
import re
from typing import List


def gen(s: str, n: int, i: int) -> str:
    r = ''
    for _ in range(n):
        r += s[i % 3]
        i //= 3
    return r


def chars(charset: str, n: int) -> List[str]:
    return [gen(charset, k, i) for k in range(n + 1) for i in range(len(charset) ** k)]


dp = [{''}, {'a', 'b', 'c'}]

for i in range(2, 6):
    dp.append(set())
    dp[i].update(f'({e})' for e in dp[i - 2])
    dp[i].update(a + c for a in dp[i - 1] for c in '+*?' if a[-1] not in '+*?')
    for j in range(1, i):
        dp[i].update(a + b for a in dp[i - j] for b in dp[j])
    for j in range(1, i - 1):
        dp[i].update(f'{a}|{b}' for a in dp[i - j - 1] for b in dp[j])

rs = [b for a in dp for b in a]
cs = chars('abc', 6)

for r in rs:
    rx = re.compile(r)
    for c in cs:
        m = 0 if rx.fullmatch(c) is None else 1
        if m or len(r) < 5 and len(c) < 5:
            print(f'{m} {r} {c}')
