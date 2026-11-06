# Compute and Interaction: Speed and Expressiveness

The engine must be both incredibly fast and elegantly easy to use. This is achieved by combining high-performance computing (SOA/SIMD) with a modern functional interaction model (Optics).

## 1. High-Performance Compute

To handle large-scale computations (aggregations, running totals, etc.), the engine employs several key strategies:

### Columnar/SOA Execution
By storing data in a **Structure of Arrays (SOA)** format, the engine ensures that computations are fundamentally optimized for modern hardware:
- **Cache Locality**: Iterating over a single column ensures that every byte fetched into the CPU cache is useful for the current calculation.
- **SIMD Vectorization**: The contiguous nature of columnar data allows the engine to use SIMD (Single Instruction, Multiple Data) instructions to process multiple values in a single clock cycle.

### Out-of-Core & Streaming
For datasets that exceed physical memory, the engine implements **Chunked, Out-of-Core processing**:
- **Chunking**: Data is partitioned into manageable blocks.
- **Streaming Pipelines**: Compute systems operate as a pipeline, pulling chunks through the execution loop, applying transformations, and sinking results to disk or a result buffer without ever needing the full dataset in RAM.

### Parallelism
The ECS architecture naturally supports parallelism. Systems that operate on disjoint sets of components can be executed concurrently across multiple CPU cores with minimal synchronization overhead.

## 2. Functional Interaction: Vectorized Optics

To provide a powerful and type-safe way to navigate N-dimensional data, we use **Vectorized Optics**. 

Traditional functional optics (Lenses, Prisms, Traversals) are often implemented as pointer-heavy, object-oriented traversals, which are catastrophically slow for large datasets. We redefine them for the columnar/ECS context.

### The Concept: Optics as Execution Plans

Instead of an optic being a function that operates on a single entity, an optic is a **Declarative Plan** that operates on entire columns.

| Optic Type | Relational/ECS Mapping | Performance Implementation |
| :--- | :--- | :--- |
| **Lens** | Accessing a specific field | A direct column-index selection. |
| **Prism** | Accessing an optional component | A bitmask-based filter applied to the column scan. |
| **Traversal** | Iterating over a set of entities | A high-speed, vectorized scan over a chunk of data. |

### Benefits
- **Expressiveness**: Users can write complex, nested queries using a clean, functional syntax.
- **Safety**: The type system ensures that queries are valid according to the component registry.
- **Performance**: Because these optics compile down to vectorized execution plans, they provide the ease of functional programming with the performance of raw, low-level array processing.
