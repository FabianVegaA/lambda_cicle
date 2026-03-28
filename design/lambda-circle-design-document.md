# λ◦ (Lambda-Circle) Design Document

**Version**: 2.0  
**Date**: 2026-03-26  
**Status**: **Revised — Ready for Phase 0 Implementation**  
**Confidence**: Medium-High (pending S5′ proof sketch)

---

## TL;DR

λ◦ is a functional programming language with linear types, traits, and automatic memory management via Lafont interaction nets. It combines a two-axis multiplicity system (`{0, 1, ω}` for quantity, `&` for observation) with a formally grounded match/view distinction and graph-structural concurrency safety. This revision resolves all critical gaps identified in the v1.0 analysis: the `&` algebra is redefined, the concurrency property is replaced with a graph-structural invariant (S5′), match/view observability is formalized, and context composition is made explicit. Implementation can begin with Phase 0 (Formal Kernel).

---

## Table of Contents

1. [Problem Statement & Constraints](#1-problem-statement--constraints)
2. [Formal Grammar](#2-formal-grammar)
3. [Multiplicity System (Revised)](#3-multiplicity-system-revised)
4. [Type System](#4-type-system)
5. [Match / View Semantics](#5-match--view-semantics)
6. [Interaction Net Translation](#6-interaction-net-translation)
7. [Concurrency Safety (S5′)](#7-concurrency-safety-s5)
8. [Trait System](#8-trait-system)
9. [Native Types](#9-native-types)
10. [Error Model](#10-error-model)
11. [Module System & Separate Compilation](#11-module-system--separate-compilation)
12. [Implementation Roadmap](#12-implementation-roadmap)
13. [Open Questions (Residual)](#13-open-questions-residual)
14. [Trade-offs](#14-trade-offs)
15. [Bibliography](#15-bibliography)

---

## 1. Problem Statement & Constraints

### Goals

Design and implement λ◦, a functional programming language that statically guarantees:

- **Type safety** — well-typed programs do not go wrong
- **Memory safety** — no use-after-free, no leaks, without garbage collection
- **Concurrency safety** — parallel subgraphs share no mutable state (by construction, not by runtime check)
- **Trait uniqueness** — deterministic method resolution with no ambiguity

### Constraints

1. Execution model: Lafont interaction nets with agents `{λ, @, δ, ε}`
2. Two-axis multiplicity system (see §3)
3. Trait uniqueness: at most one `impl C τ` per type-trait pair
4. Match/View distinction must produce observable differences in reduction traces
5. No runtime panics originating from the type system

### Success Criteria

- Type-preserving translation from source terms to interaction nets
- Memory safety verified by ε-agent semantics (no unreachable nodes at termination)
- Parallel subgraphs provably isolated via S5′ (§7)
- Deterministic trait resolution via DAG-DFS algorithm (§8)

---

## 2. Formal Grammar

> **NOTE FOR IMPLEMENTERS**: This section is normative. All other sections derive their syntax from these definitions. Resolve any ambiguity by returning here.

### 2.1 Multiplicities

```
q ::= 0        -- erased (compile-time only)
    | 1        -- owned (linear, exactly once)
    | ω        -- shared (reference-counted, many times)
    | &        -- borrowed (observer, lexically scoped)
```

`&` is a **mode annotation**, not a quantity. It cannot appear as the result of context composition (see §3). Mixing owned-quantity multiplicities (`0`, `1`, `ω`) with `&` in the same composition is a **static type error**.

### 2.2 Types

```
τ ::= Native κ             -- native base type
    | τ →^q τ              -- function type, argument used q times
    | ∀α. τ                -- universally quantified (parametric)
    | C τ                  -- trait constraint
    | μα. τ                -- inductive type (strictly positive)
    | &τ                   -- borrowed reference to τ (lifetime-erased)
```

### 2.3 Terms

```
e ::= x                          -- variable
    | λ(x :_q τ). e              -- abstraction, x used q times in e
    | e₁ e₂                      -- application
    | let x :_q τ = e₁ in e₂    -- let binding
    | match e { p₁ ↦ e₁ ... }   -- ownership-consuming pattern match
    | view e { p₁ ↦ e₁ ... }    -- observation-preserving pattern view
    | C::method(e)               -- trait method call
    | Con(e₁, ..., eₙ)          -- constructor application
    | native_lit                 -- native literal (Int, Float, Bool, Char, Unit)
```

### 2.4 Patterns

```
p ::= Con(x₁ :_q₁, ..., xₙ :_qₙ)    -- constructor pattern with field multiplicities
    | _                               -- wildcard (erases with ε-agent)
    | x                               -- variable binding
```

In `match`, fields default to multiplicity `1` (ownership transferred).  
In `view`, fields default to multiplicity `&` (observation, no ownership transfer).  
Field multiplicities may be annotated explicitly to override the default.

### 2.5 Constructors & Inductive Types

```
decl ::= data T α₁ ... αₙ where
           | Con₁ (f₁ :_q₁ τ₁) ... (fₖ :_qₖ τₖ)
           | ...
       | trait C α where
           method : τ
       | impl C τ where
           method = e
```

Inductive types must satisfy **strict positivity**: `α` must not appear in a negative position (to the left of any `→`) within `μα. τ`.

---

## 3. Multiplicity System (Revised)

### 3.1 Two-Axis Design

**V1.0 problem**: A single flat semilattice `0 ⊑ 1 ⊑ ω` with `& ⊑ ω` created undefined cases (`& + 1`, `1 + &`) and a formal category error — `&` expresses *temporality* (lexically scoped observation), not *quantity* (how many times a value is used).

**V2.0 resolution**: `&` is separated from the quantity axis. Context composition is only defined over `{0, 1, ω}`. `&` is a mode annotation on bindings and types, tracked separately by the borrow checker subsystem.

### 3.2 Quantity Semiring `{0, 1, ω}`

The quantity axis forms a commutative semiring `(Q, +, ·, 0, 1)`.

**Addition** (join — how uses combine):

| `+` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 1 | ω |
| **1** | 1 | ω | ω |
| **ω** | ω | ω | ω |

**Scalar multiplication** (how context scales under abstraction):

| `·` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 0 | 0 |
| **1** | 0 | 1 | ω |
| **ω** | 0 | ω | ω |

**Partial order**: `0 ⊑ 1 ⊑ ω` (chain). This is a complete semilattice on `Q`.

**Algebraic properties**:
- `0` is additive identity: `0 + q = q`
- `1` is multiplicative identity: `1·q = q`
- `ω` is additive absorbing: `ω + q = ω`
- `0` is multiplicative absorbing: `0·q = 0`

> **Grounding**: This semiring structure corresponds directly to the *resource semiring* in Atkey's Quantitative Type Theory (QTT, 2018), with `{0, 1, ω}` as the standard linearity semiring.

### 3.3 Borrow Mode `&`

`&` is a **mode**, not a quantity. It is governed by these rules:

- `&` may annotate any binding: `x :_& τ` means x is borrowed (observed) within a lexical scope
- `& + &` = `&` (two borrows remain a borrow — they are not consumed)
- `& + q` for `q ∈ {0, 1, ω}` = **static type error**: "cannot compose owned and borrowed contexts"
- `&·q` for any `q` = **static type error**: borrows do not scale

**Complete composition table** (✗ = static type error):

| `+` | 0 | 1 | ω | & |
|-----|---|---|---|---|
| **0** | 0 | 1 | ω | ✗ |
| **1** | 1 | ω | ω | ✗ |
| **ω** | ω | ω | ω | ✗ |
| **&** | ✗ | ✗ | ✗ | & |

**Rationale**: This matches the operational reality — you cannot transfer ownership of something you are only observing, and you cannot turn an observation into a sharing relationship without an explicit coercion (which is a design choice, not a default).

### 3.4 Subtyping

Coercions allowed by the type system (explicitly, never implicitly):

```
coerce_share  : _1 τ → _ω τ     -- upgrade owned to shared (wraps in Rc)
coerce_borrow : _1 τ → _& τ     -- borrow an owned value (creates observer wire)
coerce_borrow : _ω τ → _& τ     -- borrow a shared value (no RC increment)
```

No coercion from `&` to `1` or `ω` exists. A borrow cannot be promoted to ownership.

---

## 4. Type System

### 4.1 Typing Judgment

```
Γ ⊢ e : τ
```

Where `Γ` is a **typing context** — a finite map from variable names to `(multiplicity, type)` pairs:

```
Γ ::= ∅ | Γ, x :_q τ
```

Contexts are **split**, not shared: using a variable consumes its entry.

### 4.2 Context Operations

**Context addition** `Γ₁ + Γ₂` (pointwise, used in App rule):

```
(Γ₁ + Γ₂)(x) =
  | Γ₁(x) + Γ₂(x)   if x ∈ dom(Γ₁) ∩ dom(Γ₂)
  | Γ₁(x)            if x ∈ dom(Γ₁) \ dom(Γ₂)
  | Γ₂(x)            if x ∈ dom(Γ₂) \ dom(Γ₁)
```

Addition of multiplicities follows §3.2/§3.3. If any operand pair is `(q, &)` or `(&, q)` for `q ∈ {0,1,ω}`, the entire context addition is a type error.

**Scalar multiplication** `q·Γ` (scales all quantities):

```
(q·Γ)(x) = q · Γ(x)    for all x ∈ dom(Γ)
```

Scaling a context containing `&` bindings by any `q ∈ {0,1,ω}` is a type error. Borrow contexts do not scale.

### 4.3 Typing Rules

**Var**:
```
──────────────────────────
x :_1 τ ⊢ x : τ
```

**Var-Omega** (shared variable, may be used multiply):
```
──────────────────────────
x :_ω τ ⊢ x : τ
```

**Var-Borrow**:
```
──────────────────────────
x :_& τ ⊢ x : τ    (x is observed, not consumed)
```

**Abs** (linear argument):
```
Γ, x :_q τ₁ ⊢ e : τ₂
─────────────────────────────────
Γ ⊢ λ(x :_q τ₁). e : τ₁ →^q τ₂
```

**App**:
```
Γ₁ ⊢ e₁ : τ₁ →^q τ₂     Γ₂ ⊢ e₂ : τ₁
────────────────────────────────────────────
Γ₁ + q·Γ₂ ⊢ e₁ e₂ : τ₂
```

> The argument context is scaled by `q`: if the function uses its argument `q` times, the context resources needed to produce the argument are also needed `q` times.

**Let**:
```
Γ₁ ⊢ e₁ : τ₁     Γ₂, x :_q τ₁ ⊢ e₂ : τ₂
────────────────────────────────────────────────
Γ₁ + q·Γ₂ ⊢ let x :_q τ₁ = e₁ in e₂ : τ₂
```

**Weaken** (drop an unused binding, fires ε-agent):
```
Γ ⊢ e : τ     x ∉ dom(Γ)
──────────────────────────────────
Γ, x :_0 τ' ⊢ e : τ
```

**Contract** (share a linear binding, fires δ-agent):
```
Γ, x :_ω τ ⊢ e : τ'
────────────────────────────────
Γ, x :_1 τ ⊢ e[x/x, x/x] : τ'    (explicit: use x twice)
```

> In practice, multiplicity `ω` on a binding permits arbitrary re-use; `1` forbids it. The δ-agent is inserted by the translator, not explicitly by the programmer.

### 4.4 Metatheory (Proof Obligations)

The following must be proven before claiming the type system is sound. These are **implementation prerequisites for Phase 0**.

**Lemma 1 (Substitution)**: If `Γ₁, x :_q τ₁ ⊢ e : τ₂` and `Γ₂ ⊢ v : τ₁`, then `Γ₁ + q·Γ₂ ⊢ e[v/x] : τ₂`.

> Proof sketch: By induction on the derivation of `Γ₁, x :_q τ₁ ⊢ e : τ₂`, using the scaling property of context multiplication at the App case.

**Theorem 1 (Preservation)**: If `Γ ⊢ e : τ` and `e →_β e'`, then `Γ ⊢ e' : τ`.

> Proof sketch: By induction on `e →_β e'`, applying Lemma 1 at the β-reduction case.

**Theorem 2 (Progress)**: If `⊢ e : τ` (empty context), then either `e` is a value or there exists `e'` with `e →_β e'`.

> Proof sketch: By canonical forms — values of type `τ →^q τ'` are λ-abstractions; values of inductive type are constructor applications. Non-value terms always have a redex.

**Theorem 3 (Linearity Invariant)**: If `x :_1 τ ∈ Γ` and `Γ ⊢ e : τ'`, then `x` appears exactly once as a free variable in `e` (counting only term positions, not type annotations).

> Proof sketch: By induction on typing derivation, tracking variable occurrences through context splitting.

---

## 5. Match / View Semantics

### 5.1 Typing Rules

**Match** (ownership-consuming):

```
Γ₀ ⊢ e : T α₁...αₙ
data T where | Cᵢ (f₁ :_1 τ₁) ... (fₖ :_1 τₖ)
∀i. Γᵢ, f₁ :_1 τ₁, ..., fₖ :_1 τₖ ⊢ eᵢ : τ
patterns are exhaustive
──────────────────────────────────────────────────────────────────
Γ₀ + Γ₁ + ... + Γₙ ⊢ match e { C₁(f̄₁) ↦ e₁ | ... | Cₙ(f̄ₙ) ↦ eₙ } : τ
```

Fields introduced with multiplicity `1`: ownership is transferred to the arm body.

**View** (observation-preserving):

```
Γ₀ ⊢ e : T α₁...αₙ
data T where | Cᵢ (f₁ :_q₁ τ₁) ... (fₖ :_qₖ τₖ)
∀i. Γᵢ, f₁ :_& τ₁, ..., fₖ :_& τₖ ⊢ eᵢ : τ
patterns are exhaustive
──────────────────────────────────────────────────────────────────
Γ₀ + Γ₁ + ... + Γₙ ⊢ view e { C₁(f̄₁) ↦ e₁ | ... | Cₙ(f̄ₙ) ↦ eₙ } : τ
```

Fields introduced with multiplicity `&`: the original value `e` remains live after the `view` expression.

### 5.2 Operational Distinction (Observable in Reduction Traces)

**Match translation**: The original value node is immediately connected to an ε-agent.  
When the match arm fires, the ε-interaction eliminates the original node at that point.

```
⟦match e { C(x) ↦ body }⟧ =
  let n = ⟦e⟧
  let (x_port, _rest) = destruct(n)    -- fires ε on n (consumption)
  ⟦body⟧[x_port/x]
```

**View translation**: The original value node is wire-passed through. No ε-interaction fires at the binding site.

```
⟦view e { C(x) ↦ body }⟧ =
  let n = ⟦e⟧
  let x_port = observe(n)              -- auxiliary wire, n survives
  ⟦body⟧[x_port/x]
  -- n's ε-agent fires only at end of enclosing scope
```

**Observable difference**: In the interaction net reduction trace, `match` produces an `(ε ⋈ C)` interaction step at the binding site. `view` produces no such step there — the `(ε ⋈ C)` step appears only when the original value's owning scope closes. This difference is observable in: (a) reduction step counts, (b) memory profiles mid-computation, (c) interaction net graph snapshots.

### 5.3 Exhaustiveness

Both `match` and `view` require exhaustive patterns. Non-exhaustive patterns are a **compile-time error**. The exhaustiveness check proceeds by:

1. Enumerate all constructors of the scrutinee's type.
2. Verify each constructor appears in at least one arm (up to wildcard coverage).
3. Check no arm is unreachable (shadow detection).

---

## 6. Interaction Net Translation

### 6.1 Agents

| Agent | Arity | Role |
|-------|-------|------|
| `λ` | 2 (principal, body, var) | Lambda abstraction |
| `@` | 2 (principal, fun, arg) | Application |
| `δ` | 3 (principal, left, right) | Duplication (for `ω`) |
| `ε` | 1 (principal) | Erasure (for `0`, scope-end of `1`) |

### 6.2 Formal Translation `⟦·⟧`

The translation `⟦e⟧ : Term → Net` is defined inductively:

| Term | Net Construction |
|------|-----------------|
| `x` | Free port (wire to binding site) |
| `λ(x :_0 τ). e` | λ-node; x-port connected to fresh ε-agent |
| `λ(x :_1 τ). e` | λ-node; x-port wired directly to `⟦e⟧` |
| `λ(x :_ω τ). e` | λ-node; x-port connected to δ-agent feeding into `⟦e⟧` |
| `λ(x :_& τ). e` | λ-node; x-port wired directly (identity wire, no ownership agent) |
| `e₁ e₂` | @-node; principal connected to `⟦e₁⟧`, arg connected to `⟦e₂⟧` |
| `Con(e₁,...,eₙ)` | Constructor node C with n ports; each port i wired to `⟦eᵢ⟧` |
| `match e { Cᵢ(x̄ᵢ) ↦ eᵢ }` | Dispatch net (see §5.2); ε fires on original node |
| `view e { Cᵢ(x̄ᵢ) ↦ eᵢ }` | Dispatch net (see §5.2); original node wired through |

### 6.3 Interaction Rules

| Rule | Fires When | Result |
|------|------------|--------|
| `λ ⋈ @` (β-lin) | Linear λ meets @-node | Substitute body for argument; no δ |
| `λ ⋈ δ` (β-dup) | Shared λ meets δ-node | Duplicate the λ-body graph |
| `λ ⋈ ε` (β-drop) | Any λ meets ε-node | Erase body and var with ε-agents |
| `δ ⋈ δ` | Two δ-agents meet | Commute (rearrange duplication) |
| `δ ⋈ ε` | δ meets ε | Erase both branches |

### 6.4 Claimed Properties

**Confluence**: The interaction system is confluent (Church-Rosser). Proof: Lafont (1990) — each pair of agents has at most one interaction rule; no critical pairs.

**Memory Safety**: At termination, no ε-reachable node exists in the normal form. Proof obligation: show every node created during translation is eventually reached by an ε-agent along some reduction path. (Follows from the linearity invariant — every `1`-multiplicity node has exactly one ε-agent assigned at scope exit.)

**Optimality** (weakened claim): The translated net **shares subgraphs** for `ω`-multiplicity terms via δ-agents, avoiding re-evaluation of shared subterms. Full Lévy optimality (no redex reduced twice in the strongest sense) is **not claimed** for the initial implementation. A practical evaluator (innermost-first, left-to-right) is used. Optimality is deferred to a later optimization phase.

---

## 7. Concurrency Safety (S5′)

### 7.1 Problem with V1.0 S5

V1.0's S5 stated: "if `e₁ ⊗_par e₂` arises from β-dup, then `FV(e₁) ∩ FV(e₂) = ∅`."

This is **syntactic** and insufficient: disjoint free variables do not prevent mutable state sharing through aliasing, heap allocation, or reference types.

### 7.2 Revised Property S5′ (Graph-Structural)

**Definition (δ-root)**: A δ-agent `d` in a net `N` is a *root δ-agent* of subgraph `G ⊆ N` if `d`'s principal port is connected to a node outside `G` and at least one auxiliary port is connected to a node inside `G`.

**S5′ (Parallel Isolation)**: If `e₁ ⊗_par e₂` arises from a `(λ ⋈ δ)` interaction (β-dup), then the subgraphs `⟦e₁⟧` and `⟦e₂⟧` share **no root δ-agents**. That is:

```
root-δ-agents(⟦e₁⟧) ∩ root-δ-agents(⟦e₂⟧) = ∅
```

**Why this is sufficient**: δ-agents are the exclusive mechanism by which `ω`-multiplicity (shared, potentially mutable) values are duplicated. If two parallel subgraphs share no root δ-agents, neither can access a shared value that is also being modified by the other. Immutable values (`1`-multiplicity) are consumed and not shared; borrowed values (`&`) are read-only by definition. Therefore: **no mutable state is shared between parallel subgraphs**, satisfying R3.

**Verifiability**: S5′ is verifiable at **compile time** during net construction. After translation, the compiler walks the net and checks that no δ-agent has auxiliary ports in two distinct parallel subgraphs. This is O(|N|) in net size.

### 7.3 Runtime Parallel Execution

When S5′ holds, subgraphs `⟦e₁⟧` and `⟦e₂⟧` may be reduced in parallel on separate threads with no synchronization required. The implementation uses a **work-stealing thread pool** with per-thread reduction queues. No locks are needed between isolated subgraphs.

---

## 8. Trait System

### 8.1 Global Implementation Registry

```
Σ : TraitName × Type → Implementation
```

`Σ` is built at **link time** (see §11). Coherence (R4) requires:

```
∀ C, τ : at most one entry (C, τ) ∈ dom(Σ)
```

Duplicate implementations are a **link-time error**.

### 8.2 Typing Rules

**Trait-Use** (method call):

```
Γ ⊢ e : τ     (C, τ) ∈ Σ     method : τ_m ∈ impl C τ
──────────────────────────────────────────────────────────
Γ ⊢ C::method(e) : τ_m
```

**Trait-Abs** (trait-bounded abstraction):

```
Γ, x :_q (C α) ⊢ e : τ    α fresh
─────────────────────────────────────────────
Γ ⊢ Λα. λ(x :_q C α). e : ∀α. C α →^q τ
```

**Trait-App** (trait-bounded application):

```
Γ ⊢ e : ∀α. C α →^q τ     τ' concrete     (C, τ') ∈ Σ
──────────────────────────────────────────────────────────
Γ ⊢ e [τ'] : C τ' →^q τ[τ'/α]
```

### 8.3 Resolution Algorithm

Trait method resolution is **deterministic** and uses depth-first search over the supertrait DAG:

```
resolve(C, τ, Σ):
  1. If (C, τ) ∈ Σ: return Σ(C, τ)           -- direct impl
  2. For each C' in supertraits(C) (DFS order):
       if (C', τ) ∈ Σ: return Σ(C', τ)        -- inherited impl
  3. Error: no implementation found for C on τ
```

**Termination**: Guaranteed because the supertrait relation is a DAG (acyclic, per the trait declaration requirement).

**Determinism**: The DFS order over the DAG is fixed by declaration order (depth-first, left-to-right among supertraits). No two paths can produce different results because coherence (§8.1) forbids overlapping implementations.

### 8.4 Trait Inheritance

Trait inheritance forms a DAG declared at definition time:

```
trait C₂ : C₁ where   -- C₂ inherits from C₁
  method2 : τ
```

Implementing `C₂ τ` does **not** automatically satisfy `C₁ τ`. Both must be explicitly implemented. Inherited methods from `C₁` are accessible when `C₂` is in scope, via the resolution algorithm above.

Default method implementations are **not supported** in v1.0. This is a deliberate simplicity choice.

### 8.5 Static vs. Dynamic Dispatch

**Static dispatch only** in v1.0. Trait objects (dynamic dispatch, existential `∃τ. C τ`) are **deferred**. All trait resolution is completed at compile time.

If heterogeneous collections are needed, the programmer must use sum types (inductive `data`) explicitly.

---

## 9. Native Types

### 9.1 Supported Native κ Values (v1.0)

| Kind `κ` | Description | Multiplicity | Representation |
|----------|-------------|--------------|----------------|
| `Int` | 64-bit signed integer | `ω` (copy) | Register (64-bit) |
| `Float` | 64-bit IEEE 754 | `ω` (copy) | Register (64-bit) |
| `Bool` | Boolean | `ω` (copy) | Register (1-bit logical) |
| `Char` | Unicode scalar value | `ω` (copy) | Register (32-bit) |
| `Unit` | Zero-information value | `0` (erased) | No representation |

All native types carry multiplicity `ω` (they are copyable by value). They are stored in registers and have **no heap representation** in the interaction net. Native literals are constants in the net, not nodes.

`Unit` has multiplicity `0`: it carries no runtime information and is always erased by the compiler.

### 9.2 Native Type Interaction with Nets

Native values do not participate in ε-agent or δ-agent interactions. They are treated as **opaque leaves** in the interaction net graph. Arithmetic and comparison operations on natives are handled by a separate **primitive reduction system** that fires before interaction net steps.

### 9.3 Foreign Function Interface (FFI)

FFI is **deferred to v2.0**. Native types are the boundary between λ◦ and the outside world in v1.0. A `Foreign κ` kind for calling external code will be specified separately to avoid contaminating the net semantics.

---

## 10. Error Model

### 10.1 Compile-Time Errors (All Static)

The following are **always detected at compile time**. No runtime panic originates from the type system.

| Error | Trigger |
|-------|---------|
| `LinearityViolation` | A `_1` variable used ≠ 1 times |
| `BorrowContextMix` | `& + q` for `q ∈ {0,1,ω}` in context composition |
| `OwnershipEscape` | `&` value referenced outside its lexical scope |
| `TraitNotFound` | `C::method(e)` with no `(C, τ) ∈ Σ` |
| `DuplicateImpl` | Two `impl C τ` for same `(C, τ)` |
| `NonExhaustivePattern` | `match`/`view` missing constructor cases |
| `MultiplicityMismatch` | Expected `_q` annotation conflicts with inferred |
| `StrictPositivityViolation` | Inductive type with negative occurrence |
| `UndefinedTrait` | `impl C τ` for undeclared trait `C` |

### 10.2 Link-Time Errors

| Error | Trigger |
|-------|---------|
| `CoherenceViolation` | Duplicate `impl C τ` across modules |
| `MissingImpl` | Trait method call resolves to no implementation at link time |

### 10.3 Runtime Behavior

Well-typed λ◦ programs have **no undefined behavior**. The only runtime conditions:

- **Stack overflow** from unbounded recursion: terminates with a diagnostic (not UB)
- **Resource exhaustion** (OOM): terminates with a diagnostic (not UB)

There are no null pointers, no use-after-free, no data races — by construction from the type system and S5′.

---

## 11. Module System & Separate Compilation

### 11.1 Sealed-World Assumption

In v1.0, coherence is checked at **link time across all modules**. This is the *sealed-world assumption*: all modules that will ever be linked together are known at link time.

**Rationale**: The orphan instance problem (a coherence violation introduced by separately-compiled third-party modules) is avoided by requiring all implementations to be present at link time. This is the Haskell approach and is simpler than the Rust approach (which requires orphan rules enforced per-crate).

**Implication**: Dynamic loading of modules (plugins) is not supported in v1.0.

### 11.2 Import/Export

```
module M where
  import M₁        -- import all public names from M₁
  import M₂ (f, g) -- selective import

export (T, C, f)    -- explicit export list
```

Implementations (`impl`) are **never explicitly exported** — they are globally visible to the linker for coherence checking. Trait implementations are ambient.

### 11.3 Compilation Units

Each source file is one module. Compilation proceeds:

```
Per-module: parse → type-check → net-translate → emit .λo object
Link:        collect all .λo → coherence check → link → executable
```

Type checking is per-module (using exported signatures). Coherence checking is global (requires all modules). This allows parallel per-module compilation.

---

## 12. Implementation Roadmap

### Phase 0: Formal Kernel (Prerequisite — must complete before any code)

Goal: Establish a verified metatheory for the sublanguage `λ◦₀` with only multiplicities `{0, 1, ω}`, no traits, no native types, and no match/view (only variable binding and application).

Deliverables:
1. Prove Lemma 1 (Substitution) for `λ◦₀`
2. Prove Theorem 1 (Preservation) for `λ◦₀`
3. Prove Theorem 2 (Progress) for `λ◦₀`
4. Prove Theorem 3 (Linearity Invariant) for `λ◦₀`
5. Define `⟦·⟧` for `λ◦₀` and prove type-preservation of translation

This phase produces a paper proof (or Agda/Coq mechanization). No executable code yet.

### Phase 1: Core Type System

1. Define complete multiplicity semiring implementation
2. Parse source grammar (§2)
3. Implement multiplicity checking with context splitting
4. Implement context composition `Γ + q·Δ` (§4.2)
5. Implement linearity checking (exactly-once for `_1`)
6. Implement borrow scope checker (lexical scope exit detection)
7. Implement exhaustiveness checking for `match`/`view`
8. Implement positivity checking for inductive types
9. Error reporting for all static errors (§10.1)

### Phase 2: Interaction Net Runtime

1. Graph representation (port-and-wire model)
2. Implement four agents: `λ`, `@`, `δ`, `ε`
3. Implement five interaction rules (§6.3)
4. Implement practical evaluator (innermost-first, non-optimal)
5. Implement `⟦·⟧` translation for all term forms (§6.2)
6. Implement ε-agent memory release
7. Implement S5′ verification pass (δ-root isolation check)
8. Basic work-stealing parallel executor for isolated subgraphs

### Phase 3: Trait System

1. Implement global registry `Σ`
2. Implement coherence checking at link time
3. Implement DAG-DFS resolution algorithm (§8.3)
4. Implement trait-bounded typing rules (§8.2)
5. Module system: import/export, compilation units (§11)

### Phase 4: Optimization & Tooling

1. Optimal evaluation (Lévy sharing) — requires research investment
2. Native type primitive reduction system
3. Better diagnostics (source locations, type error explanations)
4. REPL for interactive evaluation
5. FFI design (`Foreign κ`) — v2.0 specification

---

## 13. Open Questions (Residual)

The following questions from v1.0 remain **unresolved** and are not blockers for Phases 0–2, but must be resolved before Phase 3 or 4.

### Medium Priority

**Q1: Trait objects / dynamic dispatch**  
Static resolution is sufficient for v1.0. If dynamic dispatch is needed (heterogeneous collections, plugin systems), it requires specifying existential types `∃τ. C τ` with vtable layout. Deferred to v2.0.

**Q2: Default method implementations in traits**  
Not supported in v1.0 (§8.4). If needed, requires specifying inheritance semantics and coherence rules for defaults. Deferred.

**Q3: Optimal Lévy evaluation**  
Practical evaluation is used in v1.0 (§6.4). True Lévy optimality requires a sharing graph implementation. Deferred to Phase 4 pending benchmarks showing it is needed.

**Q4: Non-lexical lifetimes for `&`**  
v1.0 uses lexical scope for borrow lifetimes (simpler). Non-lexical lifetimes (as in Rust's NLL) would allow more programs to typecheck. Deferred.

### Resolved Questions (from v1.0)

| Question | Resolution |
|----------|------------|
| `& + 1` = ? | Type error — borrows and owned values are orthogonal dimensions (§3.3) |
| R3 vs S5 | Replaced by graph-structural S5′ (§7.2) |
| Observable match/view | ε-firing site in reduction trace (§5.2) |
| `Γ + q·Δ` definition | Pointwise formula (§4.2) |
| Trait resolution algorithm | DAG-DFS (§8.3) |
| Native κ values | {Int, Float, Bool, Char, Unit} (§9.1) |
| Separate compilation | Sealed-world at link time (§11.1) |
| Error handling model | All type violations = compile errors; no type-system runtime panics (§10) |

---

## 14. Trade-offs

| Decision | Choice | Rationale | Alternative Considered |
|----------|--------|-----------|----------------------|
| `&` as mode vs. quantity | Mode (orthogonal axis) | Avoids undefined `& + 1`; matches operational semantics | Flat semilattice (v1.0 — rejected: creates type errors) |
| Concurrency guarantee | Graph-structural (S5′) | Compile-time verifiable; stronger than syntactic disjointness | Runtime checks (rejected: defeats static safety goal) |
| Trait coherence | Link-time sealed world | Simpler than per-module orphan rules | Rust-style orphan rules (deferred to v2.0) |
| Optimal evaluation | Practical first | Lévy optimality too complex for v1.0 | True Lévy (deferred to Phase 4) |
| Dynamic dispatch | Static only | Matches uniqueness requirement; simpler | Trait objects (deferred to v2.0) |
| Borrowed lifetimes | Lexical only | Simpler implementation and proof theory | Non-lexical lifetimes (deferred) |
| FFI | Deferred | Avoids contaminating net semantics in v1.0 | Early FFI (rejected: interaction with nets underspecified) |
| Default trait methods | Not supported v1.0 | Coherence across defaults is complex | Supported (deferred) |
| Memory management for ω | Reference counting | Well-understood; compatible with ε-agents for linear values | Tracing GC (rejected: incompatible with interaction net semantics) |

---

## 15. Bibliography

### Primary References

1. **Atkey, R.** — "Syntax and Semantics of Quantitative Type Theory" (LICS 2018). *Primary grounding for the {0, 1, ω} semiring as a resource semiring.*
2. **Lafont, Y.** — "Interaction Nets" (POPL 1990). *Foundational paper for the execution model.*
3. **Girard, J.-Y.** — "Linear Logic" (Theoretical Computer Science, 1987). *Theoretical basis for linear types and the distinction between `!` (sharing) and linear use.*
4. **Wadler, P.** — "Linear Types Can Change the World!" (IFIP TC 2, 1990). *Practical motivation for linear types in programming languages.*
5. **Tofte, M. & Talpin, J.-P.** — "Region-Based Memory Management" (Information and Computation, 1997). *Background for lexically-scoped memory (related to `&` borrow semantics).*

### Supporting References

6. **Lévy, J.-J.** — "Optimal Reductions in the Lambda-Calculus" (1980). *Basis for the optimality claim (weakened in v2.0 implementation).*
7. **Guzmán, J. & Hudak, P.** — "Single-Threaded Polymorphic Lambda Calculus" (POPL 1990). *Related work on linear polymorphism.*
8. **Honda, K. et al.** — "Multiparty Session Types" (POPL 2008). *Related work on communication safety — relevant if session types are added later.*
9. **The Rust Reference** — Ownership and Borrowing. *Engineering reference for `&`-mode semantics and borrow checking.*
10. **Yallop, J. & White, L.** — "Lightweight Higher-Kinded Polymorphism" (FLOPS 2014). *Reference for implementing trait-bounded polymorphism.*

---

## Appendix A: Type-Driven Implementation Guide (Haskell Pseudocode)

```haskell
-- Multiplicities
data Quantity = Zero | One | Omega
  deriving (Eq, Ord, Show)

data Multiplicity = Qty Quantity | Borrow
  deriving (Eq, Show)

-- Semiring operations on Quantity
addQ :: Quantity -> Quantity -> Quantity
addQ Zero q    = q
addQ q    Zero = q
addQ One  One  = Omega
addQ _    _    = Omega

mulQ :: Quantity -> Quantity -> Quantity
mulQ Zero _    = Zero
mulQ _    Zero = Zero
mulQ One  q    = q
mulQ q    One  = q
mulQ Omega _   = Omega
mulQ _    Omega = Omega

-- Context composition (fails on mixed Qty/Borrow)
addMul :: Multiplicity -> Multiplicity -> Either TypeError Multiplicity
addMul (Qty q1) (Qty q2) = Right (Qty (addQ q1 q2))
addMul Borrow   Borrow   = Right Borrow
addMul _        _        = Left (BorrowContextMix)

-- Types
data Type
  = TNative NativeKind
  | TArrow Multiplicity Type Type
  | TTrait TraitName Type
  | TInductive TypeName [TypeVar] [Constructor]
  | TForall TypeVar Type
  | TBorrow Type                  -- &τ
  deriving (Eq, Show)

data NativeKind = KInt | KFloat | KBool | KChar | KUnit
  deriving (Eq, Show)

-- Context
newtype Context = Context (Map Name (Multiplicity, Type))

-- Context operations
ctxAdd :: Context -> Context -> Either TypeError Context
ctxScale :: Quantity -> Context -> Either TypeError Context
ctxScale q (Context m) = traverse scaleEntry m >>= Right . Context
  where
    scaleEntry (Borrow, t) = Left BorrowContextMix
    scaleEntry (Qty qv, t) = Right (Qty (mulQ q qv), t)

-- Errors
data TypeError
  = LinearityViolation Name Multiplicity
  | BorrowContextMix
  | OwnershipEscape Name
  | TraitNotFound TraitName Type
  | DuplicateImpl TraitName Type
  | NonExhaustivePattern
  | MultiplicityMismatch Multiplicity Multiplicity
  | StrictPositivityViolation TypeName
  deriving (Show)
```

---

*Document Version: 2.0*  
*Revised: 2026-03-26*  
*Supersedes: lambda-circle-design-document v1.0*  
*Status: Ready for Phase 0 (Formal Kernel)*
