
# ssbee

_When it comes to **s**tate **s**eparating proofs, it's a busy **bee**!_

`ssbee` is a tool that helps you manage the tedious parts of working with the State-Separation Proofs framework for doing crypto proofs.

:::danger
This project is in early alpha. Expect insufficient documentation and  bugs, bugs, bugs!
:::

## Features

- Handle packages, games and proofs in a custom language close to pseudocode
- Type-check oracle code and wiring between packages
- Check that reduction game hops are valid
- Use SMT solvers to show equivalence of games with different code
  - This requires hand-writing invariants in SMT-LIB, but not proving them.
- Generate LATeX cryptocode and diagrams

## Usage

You need a somewhat recent Rust toolchain. If you don't have that, look into rustup.

Install the tool cloning the repo and running `cargo run --package ssbee`. Ensure that the installed binary is in your `PATH`.

Enter a project directory and run `ssbee prove`. To get an idea how a projects is structured, see the `example-projects` directory (documentation is on the roadmap...).

## Model

At the lowest level, there are _packages_, which can expose oracles (exports) and call oracles on other packages (imports). A package has both _state_ and _constant parameters_. One layer higher there are _games_, which instantiate packages into package instance and assign which oracle is called for every import. A game also has constant parameters that can be assigned to package constant parameters during instantiation. At the highest layer there are proofs, which instantiate games and describe hops between these. There are reduction game hops, which are graph-based arguments based on an assumption, and equivalence game hops, where we use an SMT solver to show that two games behave identically.

## Roadmap

- [ ] Documentation, Cleanup, ... - get ready for a proper release
- [ ] Type parameters - in instantiations, allow not just assigning constants, but also types.
- [ ] For loops in oracle code - so far we don't have loops. This makes the equivalence proofs more tricky.
- [ ] $n$-wise parallel composition - this is like "for loops in games".
- [ ] Automatically find invariants for equivalence proofs.
