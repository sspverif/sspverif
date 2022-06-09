# Introduction

SSPVerif is a project aiming to use formal methods to help with [State-Separating Proofs].

Code is defined in a pseudocode-like language. The tool generates SMT-LIB 2.6 code, which can be used to prove statements about the code. We are mostly interested in proving equal behaviour of two programs (in order to justify a game-hop).

We also want to type-check our code and draw composition graphs as well as generate LaTeX code from it.

[State-Separating Proofs]: https://eprint.iacr.org/2018/306
