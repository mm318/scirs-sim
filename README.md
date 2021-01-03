# scirs-sim

![Build & Test](https://github.com/mm318/scirs-sim/workflows/Build%20and%20Test/badge.svg?event=schedule)

A Rust implementation of a framework for modeling systems (similar to Simulink, Modelica, SystemModeler, etc.)

### Requirements
- Rust 2018

### Usage
To run the trivial example, execute the following command:

```
cargo test --test example -- --nocapture
```

### TODO
- Parallel execution
- Better type safety
- Reduce amount of copying from one block to the next
- More ergonomic definition of blocks
