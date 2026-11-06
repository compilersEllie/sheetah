# Architecture Overview: Multidimensional Projection Engine

## The Core Vision

The project moves beyond the traditional 2D spreadsheet model. Instead of treating data as a collection of flat tables, we treat it as a multidimensional universe of **Entities** and **Components**. 

A 2D "Table" is not the fundamental data structure; it is a **Projection**—a specific mathematical view $(\text{Row}, \text{Column})$ of the underlying entity-component set. This allows the same data to be represented simultaneously as:
- **Relational Tables** (for editing and standard data entry)
- **High-Dimensional Charts** (for analysis)
- **Entity Forms** (for single-entity focus)
- **Custom Views** (for domain-specific workflows)

## Key Architectural Pillars

### 1. Entity-Component-System (ECS) Foundation
We use an ECS model to manage data identity and logic:
- **Entities**: Stable, unique identifiers.
- **Components**: Pure, orthogonal data structures (POD).
- **Systems**: Declarative compute logic that operates on entity sets.

### 2. Adaptive Tiered Storage
To balance high-performance compute with memory efficiency, the engine uses a **Tiered Columnar (SOA) Storage** strategy:
- **Dense Tier**: High-population components are stored in contiguous, dense arrays to maximize SIMD and cache-line efficiency.
- **Sparse Tier**: Low-population components are automatically transitioned to compressed/sparse formats (e.g., Bitmaps, Compressed Indices) to avoid the "Zero-Iteration Penalty."
- **Archetype-Based Organization**: Entities are grouped by their component combinations (Archetypes), ensuring that similar data is stored together for maximum cache locality.

### 3. The Projection Engine
The engine performs "Projections" by applying a transformation function $P$ that maps N-dimensional coordinates to a 2D plane. This engine is:
- **Declarative**: Users define *what* they want to see; the engine decides *how* to fetch it.
- **Vectorized**: Projections are implemented as high-speed "Gather" operations over columnar arrays.
- **Out-of-Core**: Designed to handle datasets larger than available memory via chunked, streaming, and external-memory algorithms.

### 4. Functional Interface via "Vectorized Optics"
Users interact with the multidimensional space through a functional API. By using **Vectorized Optics** (Lenses, Prisms, and Traversals), the engine translates high-level, expressive queries into highly optimized, low-level execution plans that operate on columns rather than individual objects.
