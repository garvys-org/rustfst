# rustfst-python

## Introduction

Rust implementation of Weighted Finite States Transducers.

*Rustfst* is a library for constructing, combining, optimizing, and searching weighted
finite-state transducers (FSTs). Weighted finite-state transducers are automata where
each transition has an input label, an output label, and a weight.
The more familiar finite-state acceptor is represented as a transducer
with each transition's input and output label equal. Finite-state acceptors
are used to represent sets of strings (specifically, regular or rational sets);
finite-state transducers are used to represent binary relations between pairs of
strings (specifically, rational transductions). The weights can be used to represent
the cost of taking a particular transition.

FSTs have key applications in speech recognition and synthesis, machine translation,
optical character recognition, pattern matching, string processing, machine learning,
information extraction and retrieval among others. Often a weighted transducer is used to
represent a probabilistic model (e.g., an n-gram model, pronunciation model). FSTs can be
optimized by determinization and minimization, models can be applied to hypothesis sets
(also represented as automata) or cascaded by finite-state composition, and the best
results can be selected by shortest-path algorithms.

![fst](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/project_in.svg?sanitize=true)

## Naming

- `rustfst` is the Rust re-implementation of `Openfst`.
- `rustfst-python` is a python biding on top of `rustfst`.

## References

Implementation heavily inspired from Mehryar Mohri's, Cyril Allauzen's and Michael Riley's work :

- [Weighted automata algorithms](https://cs.nyu.edu/~mohri/pub/hwa.pdf)
- [The design principles of a weighted finite-state transducer library](https://core.ac.uk/download/pdf/82101846.pdf)
- [OpenFst: A general and efficient weighted finite-state transducer library](https://link.springer.com/chapter/10.1007%2F978-3-540-76336-9_3)
- [Weighted finite-state transducers in speech recognition](https://repository.upenn.edu/cgi/viewcontent.cgi?article=1010&context=cis_papers)
