# Data Model: Entities, Components, and Archetypes

The data model is designed to bridge the gap between mathematical multidimensionality and high-performance physical storage.

## 1. Entities and Components

The fundamental unit of data is the **Entity**. An entity does not possess data itself; instead, it is a unique identifier that "owns" a set of **Components**.

### Components
Components are the building blocks of the model. They are:
- **Orthogonal**: Each component represents a single, independent aspect of an entity (e.g., `Position`, `Velocity`, `Color`).
- **Pure Data**: They contain no logic, only raw values (POD - Plain Old Data).
- **Composable**: Entities are defined by the specific set of components they possess.

## 2. Archetypes

To achieve high performance, the engine uses **Archetypes**. An Archetype is a unique combination of component types.

### Storage Optimization
All entities belonging to the same Archetype are stored together in contiguous memory blocks. This grouping provides two critical benefits:
1. **Cache Locality**: When a system iterates through an archetype, the CPU accesses components in a predictable, sequential manner.
2. **SIMD Efficiency**: Contiguous arrays within an archetype allow for the use of Single Instruction, Multiple Data (SIMD) instructions, enabling massive parallelization of compute tasks.

## 3. Adaptive Tiered Storage

A major challenge in multidimensional models is the "Density vs. Sparsity" trade-off. We resolve this using **Adaptive Tiered Storage**.

Rather than choosing between dense or sparse storage globally, the engine monitors the density of each component within each archetype and dynamically applies the optimal storage strategy.

### The Tiers

| Tier | Best For... | Implementation | Benefit |
| :--- | :--- | :--- | :--- |
| **Dense** | High-population components | Contiguous, uncompressed arrays (SOA) | Maximum SIMD/Cache speed. |
| **Sparse** | Low-population components | Bitmaps, Compressed Indices, or Run-Length Encoding (RLE) | Minimizes memory waste and "Zero-Iteration" overhead. |

### Dynamic Transitions
The engine manages the "Promotion" and "Demotion" of components between tiers. As an archetype grows or its component distribution changes, the engine can transition storage formats to maintain the optimal balance between memory footprint and compute throughput.

## 4. Schema and Versioning

The schema is an emergent property of the components present in the system.
- **Component Registry**: A central, versioned registry defines all available component types and their fields.
- **Evolution**: Adding new components is a non-breaking operation. Modifying existing components is handled by treating the modification as a new component version, allowing for smooth data migrations.
