# nu_plugin_from_beancount

[![License](https://img.shields.io/crates/l/nu_plugin_from_beancount)](#Unlicense)
[![Crates.io](https://img.shields.io/crates/v/nu_plugin_from_beancount)](https://crates.io/crates/nu_plugin_from_beancount)
![rustc](https://img.shields.io/badge/rustc-1.62+-blue?logo=rust)

A [nushell] extension convert [beancount] file into nu structured data.

When using [nushell] one could run `bean-example | from beancount` or `open my-ledger.beancount` to get a nushell structured
table that can be further transformed and manipulated.

[nushell]: https://www.nushell.sh/
[beancount]: https://beancount.github.io/docs/index.html


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

* [x] Transaction
  * [x] Postings
    * [x] Account
    * [x] Amount
    * [ ] Cost
* [x] Include directive
* [ ] Pad directive
* [ ] Balance assertion
* [ ] Option
* [ ] ...


## Installation

```nu
cargo install nu_plugin_from_beancount
register ~/.cargo/bin/nu_plugin_from_beancount
```

## Companion scripts

Currently, this plugin is intentionally quite dumb and does nothing more than outputing the beancount file content without any transformation or resolution.
But for most ledgers, the content of the file does not contain all informations required for further analysis.
For example, the transactions are often not explicitely balanced (one posting has no explicit amount), making it hard to make any useful aggregation of the amounts.

Here are some "companion" scripts that you may find useful when working with `from beancount`.

> **Note**
> Those scripts are provided here as a suggestion. They are not thoroughy tested, and are not considered part of the plugin API.

```nu
# Negate a structured amount (which is a record of currencies to values)
def "bean amount neg" [] {
  transpose currency value | update value { -($in.value) } | transpose -ird
}

# Complete transactions that are not explitly balanced, so that all transactions balance
def "bean resolve txn" [] {
  each {|row|
    if $row.directive != "txn" { $row } else {
      let offset_accounts = ($row.postings | where amount == $nothing | get -i account)
      if ($offset_accounts | is-empty) { $row } else {
        let complete_postings = ($row.postings | where amount != $nothing)
        let offset_account = ($offset_accounts | first)
        let offset_amount = ($complete_postings | get amount | math sum | bean amount neg)
        $row | update postings ($complete_postings | append {account: $offset_account, amount: $offset_amount})
      }
    }
  }
}
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
