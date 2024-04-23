# Folidity - Formally Verifiable Smart Contract Language

This project addresses the long-lasting problem involving the exploits of Smart Contract vulnerabilities. There are tools, such as in the formal verification field and alternative Smart Contract languages, that attempt to address these issues. However, neither approach has managed to combine the static formal verification and the generation of runtime assertions. Furthermore, this work believes that implicit hidden state transition is the root cause of security compromises. In light of the above, we introduce Folidity, a formally verifiable Smart Contract language with a unique approach to reasoning about the modelling and development of Smart Contract systems. Folidity features explicit state transition checks, a model-first approach, and built-in formal verification compilation stage.

Folidity targets [Algorand Virtual Machine](https://developer.algorand.org/docs/get-details/dapps/avm) and emits [Teal](https://developer.algorand.org/docs/get-details/dapps/avm/teal/) bytecode. The EVM support is at the planned work.  

## Installation

- Prerequisites
  - Install [`z3`](https://github.com/Z3Prover/z3).
    - Most systems that have LLVM already have it installed.
    - Otherwise, you can install it with the standard package manager (e.g. `brew install z3`)
  - Install [rust stable](https://www.rust-lang.org/learn/get-started)
- Install the binary: `cargo install --path crates/folidity`
- (Optional) install cargo nightly for formatting when developing: `rustup toolchain install nightly`

## Usage
Start with `folidity help` to get the overview of the supported command and their options.

- `folidity new ...` - Creates a new templated `folidity` counter project. with a basic contract, README and approval teal code
- `folidity check ...` - Check the contract's code for parser, semantic and type errors
- `folidity verify ...`  - Check the contract's code for errors and validate model consistency using static analysis and symbolic execution
- `foliidty compile ...` - Check the contract's code for errors and validate model consistency using static analysis and symbolic execution

## Status

Folidity is an exprimental project and is not currently considered Production Ready for general use. It may have unexpected behavior outside of the scenarios it has been used for until now.

## Changelog

The project maintains a clear [changelog](CHANGELOG.md) of versions and adheres to semantic versioning.

## License

Folidity is currently licensed under the [University of Southampton Intellectual Property Regulation](https://www.southampton.ac.uk/about/governance/regulations-policies/general-regulations/intellectual-property).

The project maintainer is at the negotiating stage to fully open-source the project under MIT/Apache 2.0 License.