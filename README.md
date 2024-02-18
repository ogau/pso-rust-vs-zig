# Comparison of PSO implementations in Rust and Zig
This project contains two implementations of the Particle Swarm Optimization (PSO) algorithm in Rust and Zig programming languages.

The goal of the project is to compare the experience of developing the same algorithm in two different languages, as well as test the performance of the implementations.

### Running
To run the Rust version:

```bash
$ cd rust
$ cargo r -r
```


To run the Zig version:
```bash
$ cd zig
$ zig build run
````

### Brief overview of PSO algorithm
The Particle Swarm Optimization (PSO) algorithm is an optimization algorithm inspired by the social behavior of bird flocks or insect swarms. In PSO there is a swarm of particles, each of which represents a possible solution to an optimization problem. The particles fly through the solution space, communicating their best found positions to each other. Over time the whole swarm converges to the optimal solution.

In this project PSO is used to optimize a simple test function.
