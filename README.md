# Rust ZK Lab

A structured learning repository focused on mastering Rust fundamentals and applying them to basic cryptographic concepts essential for zero-knowledge programming.

## Overview

This project is designed as an educational progression. It begins by covering core Rust mechanics—such as memory management, advanced abstractions, and concurrency—before culminating in a final lab that introduces foundational concepts used in zero-knowledge systems.

## Repository Structure

The modules are sequentially numbered to provide a clear learning path:

*   **s01_memory:** Covers basic and advanced memory management in Rust, including the ownership model and borrow checker.
*   **s02_abstraction:** Explores generics, trait objects, closures, and lifetimes for building flexible code.
*   **s03_smart_pointers:** Details heap allocation and interior mutability using `Box`, `Rc`, and `RefCell`.
*   **s04_concurrency:** Demonstrates safe concurrent programming with threads, synchronization primitives (`Mutex`, `Arc`), and message passing via channels.
*   **s05_zk_lab:** Applies the previously learned concepts to basic cryptographic primitives (such as hashing via `sha2`). This acts as a stepping stone toward ZK protocol engineering.

## Getting Started

To run the experiments and examples in this lab, you simply need a standard Rust toolchain.

```sh
# Clone the repository
git clone <repository-url>
cd rust-zk-lab

# Build the project
cargo build

# Execute the labs
cargo run
```

## Dependencies

The project intentionally keeps dependencies to a minimum to emphasize standard library features and language mechanics:

*   `sha2` (v0.10) - Used for hashing experiments in the ZK lab.
*   `hex` (v0.4) - Used for hexadecimal data formatting and display.
