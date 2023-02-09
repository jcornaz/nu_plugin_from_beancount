# rust-template

![rustc](https://img.shields.io/badge/rustc-1.62+-blue?logo=rust)

A nushell extension to load a beancount file into nu structured data.

The idea is that in a [nu] shell one could run `bean-example | from beancount` to get a nu-shell structured
data that could be further transformed and manipulated by the user.

[nu]: https://www.nushell.sh/

## Supported Beancount syntax

* [x] Transaction
  * [x] Flag
  * [x] Date
  * [x] Payee
  * [x] Narration
  * [x] Postings
    * [x] Account
    * [x] Amount
* [ ] Include directive
* [ ] Pad directive
* [ ] Balance assertion
* [ ] ...


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
