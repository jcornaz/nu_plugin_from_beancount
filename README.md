# nu_plugin_from_beancount

[![License](https://img.shields.io/crates/l/nu_plugin_from_beancount)](#Unlicense)
[![Crates.io](https://img.shields.io/crates/v/nu_plugin_from_beancount)](https://crates.io/crates/nu_plugin_from_beancount)
![rustc](https://img.shields.io/badge/rustc-1.62+-blue?logo=rust)
[![Docs](https://docs.rs/nu_plugin_from_beancount/badge.svg)](https://docs.rs/nu_plugin_from_beancount)

A [nushell] extension to load a beancount file into nu structured data.

When using [nushell] one could run `bean-example | from beancount` or `open myledger.beancount` to get a nushell structured
table that can be further transformed and manipulated.

[nushell]: https://www.nushell.sh/


## Example

```nu
> bean-example | from beancount | where direcive == txn | take 1
╭───┬─────────────┬───────────┬──────┬───────┬──────────────────────────────────────┬────────────────────────────────────────────────────╮
│ # │    date     │ directive │ flag │ payee │              narration               │                      postings                      │
├───┼─────────────┼───────────┼──────┼───────┼──────────────────────────────────────┼────────────────────────────────────────────────────┤
│ 0 │ 2 years ago │ txn       │ *    │       │ Opening Balance for checking account │ ╭───┬─────────────────────────┬──────────────────╮ │
│   │             │           │      │       │                                      │ │ # │         account         │      amount      │ │
│   │             │           │      │       │                                      │ ├───┼─────────────────────────┼──────────────────┤ │
│   │             │           │      │       │                                      │ │ 0 │ Assets:US:BofA:Checking │ ╭───┬─────────╮  │ │
│   │             │           │      │       │                                      │ │   │                         │ │ # │   USD   │  │ │
│   │             │           │      │       │                                      │ │   │                         │ ├───┼─────────┤  │ │
│   │             │           │      │       │                                      │ │   │                         │ │ 0 │ 4560.14 │  │ │
│   │             │           │      │       │                                      │ │   │                         │ ╰───┴─────────╯  │ │
│   │             │           │      │       │                                      │ │ 1 │ Equity:Opening-Balances │ ╭───┬──────────╮ │ │
│   │             │           │      │       │                                      │ │   │                         │ │ # │   USD    │ │ │
│   │             │           │      │       │                                      │ │   │                         │ ├───┼──────────┤ │ │
│   │             │           │      │       │                                      │ │   │                         │ │ 0 │ -4560.14 │ │ │
│   │             │           │      │       │                                      │ │   │                         │ ╰───┴──────────╯ │ │
│   │             │           │      │       │                                      │ ╰───┴─────────────────────────┴──────────────────╯ │
╰───┴─────────────┴───────────┴──────┴───────┴──────────────────────────────────────┴────────────────────────────────────────────────────╯
```


## Supported Beancount syntax

* [X] Transaction (with postings)
* [ ] Postings cost
* [ ] Include directive
* [ ] Pad directive
* [ ] Balance assertion
* [ ] Option
* [ ] ...


## Installation

```nu
cargo install nu_plugin_from_beancount
register ~/.cargo/bin/nu_plugin_from_beancount
```


## Unlicense

This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>
