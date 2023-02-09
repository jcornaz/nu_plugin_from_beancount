# How to contribute

## Ask for help, propose a feature request, a feature or report a bug

Use the [discussions](https://github.com/jcornaz/nu_plugin_from_beancount/discussions) to ask questions, share/discuss idea of features and even show-case what cool think you made with this project!

Use the [issues](https://github.com/jcornaz/nu_plugin_from_beancount/issues) to report any issue you have (bug or missing feature). Make sure to explain why you need something.


## Work with the sources

1. Make sure you have latest stable rust toolchain installed (https://rustup.rs)
2. Make sure you have [just](https://just.systems/man/en/chapter_4.html) installed
3. Run `just -l` to see the list of available recipes

## Coding standards

### Tests

***This is a test-driven project!*** Every new feature and bug fixes must come with tests.


## Open a pull request

Don't be afraid of small steps. I'd rather review 5 tiny pull-requests than 1 big. It is fine to have a PR that only partilally implement a feature. We can gate the feature behind a feature flag until it is complete.

But the no matter how small the PR is, it must have automated tests!
