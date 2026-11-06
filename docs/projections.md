# Projections: Mapping N-Dimensions to 2D

The "Projection Engine" is the heart of the system. Its role is to transform the N-dimensional entity-component universe into a usable 2D representation.

## 1. The Mathematical Model

A projection is formally defined as a mapping function $P$ that takes an entity $e$ and returns a 2D coordinate in a visual/edit space:

$$P(e) = (\text{Row}, \text{Col})$$

### Components of the Mapping
- **Row ($\text{Row}$)**: Determined by a grouping or sorting operation on a specific subset of attributes $A_{\text{row}} \subset A(e)$. For example, grouping by `Department` defines the rows.
- **Column ($\text{Col}$)**: Determined by the index of a specific attribute $a$ within an ordered set of visible columns $A_{\text{col}}$. For example, the column index for `Salary`.

## 2. Fundamental Operations

### Rotation
Rotation is the process of redefining the mapping function $P \to P'$. By swapping the roles of attributes in the $\text{Row}$ and $\text{Col}$ definitions, the user can "rotate" the view (e.g., pivoting a table).

### Slicing
Slicing is the restriction of the entity domain $E$ based on a predicate $Q$ (a filter):
$$E_{\text{slice}} = \{e \in E \mid Q(e) = \text{True}\}$$

### Filtering
Filtering is a subset operation performed on the *result* of a projection, allowing users to narrow down the visible 2D rows/columns based on their projected values.

## 3. Implementation: The Projection Engine

To maintain performance at scale, the projection engine does not "move" data. Instead, it generates **Virtual Views**.

### Vectorized "Gather"
When a projection is requested, the engine does not iterate through entities one-by-one. Instead, it performs a **Vectorized Gather** operation:
1. It identifies the required columns (the "target" attributes).
2. It uses the projection logic to identify the required indices (rows).
3. It performs high-speed, cache-friendly reads from the underlying columnar storage to populate the 2D view.

### Declarative Execution
The projection is expressed **declaratively**. Users specify the *intent* (e.g., "Group by X, Show Y and Z"), and the engine generates an optimized execution plan. This allows the engine to:
- Skip unnecessary data (via Zone Maps and Bitmaps).
- Optimize memory access patterns (via SIMD).
- Stream data in chunks for large-scale datasets.
