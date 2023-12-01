# Core BPF Program Specifications

Core BPF programs are integral components of the Solana network, relied upon
by every client no matter the implementation.

As we start to migrate native programs to Core BPF, the idea of possibly
implementing these programs in many different ways to test against each other
for parity in results as been suggested.

This folder aims to provide some conceptual tooling toward this proposed
approach.

## Overview

In order to allow for any implementation of a Core BPF program, we need to
provide two things:

- A single, source-of-truth specification for a Core BPF program, abstract from
  any implementation.
- A standalone test harness that will test a compiled program against it's
  designated specification.

## Specification

An example specification for the Address Lookup Table program can be found
in this folder at `address-lookup-table`.

TODO: We must decide on the proper declarative specification language to use,
or write our own.

## Test Harness

An implementation-agnostic test harness can be found outside of this folder in
the `test-harness` crate. It's designed to be fed a `.so` file and which spec
this program is designed to implement, and it will test it for validity.