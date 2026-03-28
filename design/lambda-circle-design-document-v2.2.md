# λ◦ (Lambda-Circle) Design Document

**Version**: 2.2  
**Date**: 2026-03-27  
**Status**: **Revised — v2.1 Corrections Applied**  
**Confidence**: Medium (formal proofs pending Phase 0)

---

## TL;DR

λ◦ is a functional programming language with linear types, traits, and automatic memory management via Lafont interaction nets. This revision applies corrections identified in the v2.1 analysis:

- **Fixed**: Contract rule removed from source type system (belongs in net translation only)
- **Fixed**: Let rule scaling corrected for `q = ω` case
- **Fixed**: Borrow-Weaken rule removed (redundant with Weaken)
- **Fixed**: View field coercion made explicit (view-coercion rule added)
- **Fixed**: View observability liveness clause added
- **Fixed**: S5′ algorithm acyclicity precondition added
- **Fixed**: `0·Γ` semantics defined as `_0`-annotated context (not empty context)
- **Fixed**: Unit type description corrected in §9 normative text
- **Added**: Immutable ω consequence documented in §14 Trade-offs
- **Added**: Phase 0 inductive proof cases enumerated explicitly

---

## Table of Contents

1. [Problem Statement & Constraints](#1-problem-statement--constraints)
2. [Formal Grammar](#2-formal-grammar)
3. [Multiplicity System](#3-multiplicity-system)
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
16. [Appendix A: Haskell Pseudocode](#16-appendix-a-haskell-pseudocode)
17. [Appendix B: Correction Log](#17-appendix-b-correction-log)

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
    | (τ₁, τ₂)             -- product type
    | τ₁ + τ₂              -- sum type
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
    | e₁ op₂ e₂                 -- binary native operation
    | op₁ e                     -- unary native operation
```

### 2.4 Native Operations

```
op₂ ::= + | - | * | / | % | == | != | < | > | <= | >= | && | ||
op₁ ::= - | !
```

All native operations take and return values of multiplicity `ω` (copyable by value). Native operations are typed separately from the interaction net type system (see §9.2).

### 2.5 Patterns

```
p ::= Con(x₁ :_q₁, ..., xₙ :_qₙ)    -- constructor pattern with field multiplicities
    | _                               -- wildcard (erases with ε-agent)
    | x                               -- variable binding
```

In `match`, fields default to multiplicity `1` (ownership transferred).  
In `view`, all fields are coerced to multiplicity `&` by the view-coercion rule (§5.1), regardless of their declared multiplicity.  
Field multiplicities may be annotated explicitly in `match` arms. Explicit `_&` annotation in a `match` arm is a **type error** (use `view` instead).

### 2.6 Declarations

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

## 3. Multiplicity System

### 3.1 Two-Axis Design

**Key distinction**:
- **Quantity axis** `{0, 1, ω}`: How many times a value can be used
- **Mode axis** `{&}`: What relationship the binding has to the value (observation vs. ownership)

These axes are **orthogonal**: a binding is either on the quantity axis or the mode axis, never both. Context composition is only defined within each axis. Cross-axis composition is a static type error.

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

**Semiring properties** (all hold; verifiable by table exhaustion):
- Additive commutativity: `q₁ + q₂ = q₂ + q₁`
- Additive associativity: `(q₁ + q₂) + q₃ = q₁ + (q₂ + q₃)`
- Additive identity: `0 + q = q`
- Multiplicative identity: `1 · q = q`
- Multiplicative associativity: `(q₁ · q₂) · q₃ = q₁ · (q₂ · q₃)`
- Left distributivity: `q₁ · (q₂ + q₃) = (q₁ · q₂) + (q₁ · q₃)`
- Right distributivity: `(q₁ + q₂) · q₃ = (q₁ · q₃) + (q₂ · q₃)`
- Additive absorption: `ω + q = ω` for all `q`
- Multiplicative zero: `0 · q = 0` for all `q`

**Partial order**: `0 ⊑ 1 ⊑ ω` (chain). Complete semilattice on `Q`.

> **Grounding**: This semiring matches the standard linearity semiring in Atkey's Quantitative Type Theory (QTT, 2018).

### 3.3 Borrow Mode `&`

`&` is a **mode**, not a quantity. Rules:

- `&` may annotate any binding: `x :_& τ` means `x` is observed within a lexical scope
- `& + & = &` (two borrows remain a borrow)
- `& + q` for `q ∈ {0, 1, ω}` = **static type error**: `BorrowContextMix`
- `& · q` for any `q` = **static type error**: borrows do not scale

**Complete composition table** (✗ = `BorrowContextMix` type error):

| `+` | 0 | 1 | ω | & |
|-----|---|---|---|---|
| **0** | 0 | 1 | ω | ✗ |
| **1** | 1 | ω | ω | ✗ |
| **ω** | ω | ω | ω | ✗ |
| **&** | ✗ | ✗ | ✗ | & |

### 3.4 Subtyping / Explicit Coercions

Coercions are always **explicit** in the surface language — never implicit:

```
coerce_share  : _1 τ → _ω τ     -- upgrade owned to shared (wraps in Rc)
coerce_borrow : _1 τ → _& τ     -- borrow an owned value (identity wire in net)
coerce_borrow : _ω τ → _& τ     -- borrow a shared value (no RC increment)
```

No coercion from `&` to `1` or `ω` exists. A borrow cannot be promoted to ownership.

---

## 4. Type System

### 4.1 Typing Judgment

```
Γ ⊢ e : τ
```

`Γ` is a **typing context** — a finite map from names to `(multiplicity, type)` pairs:

```
Γ ::= ∅ | Γ, x :_q τ
```

Contexts are **split**, not shared: using a variable consumes its entry.

### 4.2 Context Operations

**Context addition** `Γ₁ + Γ₂` (pointwise):

```
(Γ₁ + Γ₂)(x) =
  | Γ₁(x) + Γ₂(x)   if x ∈ dom(Γ₁) ∩ dom(Γ₂)
  | Γ₁(x)            if x ∈ dom(Γ₁) \ dom(Γ₂)
  | Γ₂(x)            if x ∈ dom(Γ₂) \ dom(Γ₁)
```

Multiplicity addition follows §3.2/§3.3. Any `(q, &)` or `(&, q)` pair for `q ∈ {0,1,ω}` makes the entire context addition a `BorrowContextMix` error.

**Scalar multiplication** `q · Γ` (scales all entries):

```
(q · Γ)(x) = q · Γ(x)    for all x ∈ dom(Γ)
```

> **CORRECTION (v2.2)**: `0 · Γ` is defined as a context where **every binding is annotated `_0`**, not as the empty context `∅`. This is because `_0` bindings still trigger Weaken (and insert ε-agents in the translation), making erasure explicit and traceable. The empty context `∅` is distinct from `0 · Γ`.

Scaling a context containing `&` bindings is a `BorrowContextMix` error. Borrow contexts do not scale.

**Context scaling distributes over context addition**:

```
q · (Γ₁ + Γ₂) = (q · Γ₁) + (q · Γ₂)
```

This holds pointwise from semiring distributivity (§3.2). Proof obligation: formalize for all multiplicity cases in Phase 0.

### 4.3 Typing Rules

**Var** (linear variable, consumed on use):
```
──────────────────────────
x :_1 τ ⊢ x : τ
```

**Var-Omega** (shared variable, may be used multiply):
```
──────────────────────────
x :_ω τ ⊢ x : τ
```

**Var-Borrow** (borrowed variable, observed not consumed):
```
──────────────────────────
x :_& τ ⊢ x : τ
```

**Abs**:
```
Γ, x :_q τ₁ ⊢ e : τ₂
─────────────────────────────────
Γ ⊢ λ(x :_q τ₁). e : τ₁ →^q τ₂
```

**App**:
```
Γ₁ ⊢ e₁ : τ₁ →^q τ₂     Γ₂ ⊢ e₂ : τ₁
────────────────────────────────────────────
Γ₁ + q · Γ₂ ⊢ e₁ e₂ : τ₂
```

The argument context is scaled by `q`: if the function uses its argument `q` times, the resources to produce the argument are needed `q` times.

**Let**:

> **CORRECTION (v2.2)**: The v2.1 Let rule was asymmetric — `e₁` was typed in `Γ₁` unscaled, but if `q = ω` then `x` is used many times, so `e₁`'s resources must also be available many times. The corrected rule scales `e₁`'s context by `q`:

```
q · Γ₁ ⊢ e₁ : τ₁     Γ₂, x :_q τ₁ ⊢ e₂ : τ₂
────────────────────────────────────────────────────
Γ₁ + Γ₂ ⊢ let x :_q τ₁ = e₁ in e₂ : τ₂
```

When `q = 1`: `1 · Γ₁ = Γ₁`, so the result is `Γ₁ + Γ₂` (standard linear let).  
When `q = ω`: `ω · Γ₁ = ω · Γ₁` (every binding in Γ₁ becomes `ω`), reflecting that all resources for `e₁` are needed in shared mode.  
When `q = 0`: `0 · Γ₁` (every binding becomes `_0`, all erased via Weaken).

**Weaken** (drop an unused binding; fires ε-agent in translation):
```
Γ ⊢ e : τ
──────────────────────────────────
Γ, x :_0 τ' ⊢ e : τ
```

> **CORRECTION (v2.2)**: Removed the redundant premise `x ∉ dom(Γ)`. Adding a `_0` binding is always valid — if `x` were already in `Γ`, the duplicate would be handled by the context splitting semantics, not by Weaken.

> **CORRECTION (v2.2)**: Removed the "Borrow-Weaken" rule that appeared in v2.1. It was identical to standard Weaken applied to a context that happens to contain a `&` binding. The separate rule added noise without meaning and could mislead implementers into thinking borrows require special weakening treatment.

> **NOTE on Contract**: The Contract structural rule does **not** appear in the surface type system. It is not a source-level rule. The sharing of an `ω`-annotated binding is handled implicitly by Var-Omega (which allows multiple uses) and explicitly in the interaction net translation by inserting a δ-agent at the binding site. See §6.2 for the translation. Source-level programmers express sharing by annotating a binding `_ω`; the compiler inserts δ-agents automatically.

### 4.4 Metatheory (Proof Obligations for Phase 0)

The following must be proven for the sublanguage `λ◦₀` (multiplicities `{0, 1, ω}` only, no traits, no native types, no match/view) before implementation begins.

**Lemma 1 (Substitution)**: If `Γ₁, x :_q τ₁ ⊢ e : τ₂` and `Γ₂ ⊢ v : τ₁`, then `Γ₁ + q · Γ₂ ⊢ e[v/x] : τ₂`.

**Inductive cases that must be discharged (Phase 0 scope)**:
- *Var*: If `e = x`, then `e[v/x] = v`. Need `Γ₁ + 1 · Γ₂ = Γ₁ + Γ₂ ⊢ v : τ₂`. Holds by hypothesis (Γ₁ = ∅ in the Var case).
- *Var-other*: If `e = y` for `y ≠ x`, then `e[v/x] = y`. Context contains `y :_q' τ'` unaffected by substitution. Holds trivially.
- *Abs*: If `e = λ(y :_r τ₃). e'`, need to show substitution commutes with binding. Uses freshness of `y` w.r.t. `v`.
- *App*: If `e = e₁ e₂`, the App rule splits context as `Γ_e1 + q_f · Γ_e2`. After substitution, need distributivity of context scaling (`q · (Γ_e1 + q_f · Γ_e2) = q · Γ_e1 + q · (q_f · Γ_e2)`). This is the critical case — uses semiring associativity.
- *Let*: If `e = let y :_r τ₃ = e₁ in e₂`, uses corrected Let rule. The scaling `r · Γ_e1` must compose correctly with the outer scaling `q`. Uses semiring multiplication associativity: `q · (r · Γ) = (q · r) · Γ`.
- *Weaken*: Trivial — `_0` bindings vanish under substitution.

**Theorem 1 (Preservation)**: If `Γ ⊢ e : τ` and `e →_β e'`, then `Γ ⊢ e' : τ`.

*Proof sketch*: By induction on `e →_β e'`. At the β-reduction step `(λ(x :_q τ₁). e) v → e[v/x]`, apply Lemma 1 with the App rule premises.

**Theorem 2 (Progress)**: If `⊢ e : τ` (empty context), then either `e` is a value or there exists `e'` with `e →_β e'`.

*Proof sketch*: By canonical forms. Values of type `τ₁ →^q τ₂` are λ-abstractions; values of inductive type `T` are constructor applications. Any non-value closed term has a redex.

**Theorem 3 (Linearity Invariant)**: If `x :_1 τ ∈ Γ` and `Γ ⊢ e : τ'`, then `x` appears exactly once as a free variable in `e` (term positions only, not type annotations).

*Inductive cases*: Var (x appears once, consumed), Abs (x bound or absent), App (context split ensures x in exactly one subterm), Let (corrected scaling ensures x in exactly one position).

**Theorem 4 (Type Preservation of Translation)**: If `Γ ⊢ e : τ`, then `⟦e⟧` is a well-formed net with output port of type `τ` and all ε-agents matched to nodes per the linearity invariant. (Formal statement requires defining net typing — Phase 0 deliverable.)

---

## 5. Match / View Semantics

### 5.1 Typing Rules

**Match** (ownership-consuming):

```
Γ₀ ⊢ e : T α₁...αₙ
data T where | Cᵢ (f₁ :_1 τ₁) ... (fₖ :_1 τₖ)
∀i. Γᵢ, f₁ :_1 τ₁, ..., fₖ :_1 τₖ ⊢ eᵢ : τ
patterns are exhaustive
──────────────────────────────────────────────────────────────────────
Γ₀ + Γ₁ + ... + Γₙ ⊢ match e { C₁(f̄₁) ↦ e₁ | ... | Cₙ(f̄ₙ) ↦ eₙ } : τ
```

All fields in match arms have multiplicity `1`. Ownership is transferred to the arm body.

**View** (observation-preserving):

> **CORRECTION (v2.2)**: The v2.1 view rule silently overrode all declared field multiplicities to `&` without stating this explicitly. This is now formalized as a **view-coercion rule**.

**View-Coercion Rule**: In a `view` arm, every field binding is coerced to `&` regardless of its declared multiplicity in the data type. This coercion is always valid (ownership is never required for observation) and is applied automatically by the type checker. The original value's multiplicity is unchanged.

```
Γ₀ ⊢ e : T α₁...αₙ    (e retains its multiplicity after the view)
data T where | Cᵢ (f₁ :_q₁ τ₁) ... (fₖ :_qₖ τₖ)
  -- view-coercion: all field bindings use _& regardless of declared qᵢ
∀i. Γᵢ, f₁ :_& τ₁, ..., fₖ :_& τₖ ⊢ eᵢ : τ
patterns are exhaustive
──────────────────────────────────────────────────────────────────────
Γ₀ + Γ₁ + ... + Γₙ ⊢ view e { C₁(f̄₁) ↦ e₁ | ... | Cₙ(f̄ₙ) ↦ eₙ } : τ
```

**Consequence**: A field declared `_1` in the data type appears as `_&` in a `view` arm. It looks borrowed because it is borrowed — the original value retains ownership. There is no semantic surprise here; the view-coercion rule makes this explicit.

### 5.2 Operational Distinction (Observable in Reduction Traces)

**Match translation**: The original value node is connected to an ε-agent at the match binding site.

```
⟦match e { C(x) ↦ body }⟧ =
  let n = ⟦e⟧
  let (x_port, _rest) = destruct(n)    -- fires ε ⋈ C on n (consumption)
  ⟦body⟧[x_port/x]
```

At the match binding site, the `ε ⋈ C` interaction fires. The original node is consumed before any reduction of the arm body begins.

**View translation**: The original value node is wired through (observer wire). No ε-interaction fires at the binding site.

```
⟦view e { C(x) ↦ body }⟧ =
  let n = ⟦e⟧
  let x_port = observe(n)              -- auxiliary wire only; n survives
  ⟦body⟧[x_port/x]
  -- n's ε ⋈ C fires only at scope exit of the enclosing owner
```

**Formal definition of observable difference**:

- `match`: ∃ step `ε ⋈ C` in the reduction trace **before** any reduction step of the arm body.
- `view`: Every step `ε ⋈ C` on the scrutinee node occurs **at or after** the arm body's last reduction step, **and** at least one `ε ⋈ C` step occurs at or before the enclosing scope's own ε fires.

> **CORRECTION (v2.2)**: Added the liveness clause to the `view` definition. Without it, the universal quantifier `∀ step ε ⋈ C` is vacuously satisfied when no ε-interaction occurs at all — which would make an incorrect translator (one that never frees the value) indistinguishable from a correct one.

The liveness clause guarantees: in a `view`, the scrutinee is **eventually** freed (at scope exit), not merely "deferred indefinitely." This is the operational content of `_1` multiplicity for the scrutinee.

### 5.3 Exhaustiveness

Both `match` and `view` require exhaustive patterns. Non-exhaustive patterns are a **compile-time error** (`NonExhaustivePattern`).

Exhaustiveness checking algorithm:
1. Enumerate all constructors of the scrutinee's inductive type.
2. Verify every constructor is covered by at least one arm (wildcards `_` cover all remaining).
3. Detect and reject unreachable arms (shadowed by prior arms).

---

## 6. Interaction Net Translation

### 6.1 Agents

| Agent | Arity | Role |
|-------|-------|------|
| `λ` | 3-port (principal, body, var) | Lambda abstraction |
| `@` | 3-port (principal, fun, arg) | Application |
| `δ` | 3-port (principal, left, right) | Duplication (for `ω`) |
| `ε` | 1-port (principal) | Erasure (for `0`; scope-end of `1`) |

### 6.2 Formal Translation `⟦·⟧`

The translation `⟦e⟧ : Term → Net` is defined inductively over the term grammar:

| Term | Net Construction |
|------|-----------------|
| `x` | Free port (wire to binding site) |
| `λ(x :_0 τ). e` | λ-node; var-port wired to fresh ε-agent |
| `λ(x :_1 τ). e` | λ-node; var-port wired directly to `⟦e⟧` |
| `λ(x :_ω τ). e` | λ-node; var-port connected to δ-agent, δ feeds into `⟦e⟧` at each use site |
| `λ(x :_& τ). e` | λ-node; var-port wired directly (identity wire, no ownership agent) |
| `e₁ e₂` | @-node; fun-port to `⟦e₁⟧`, arg-port to `⟦e₂⟧` |
| `Con(e₁,...,eₙ)` | Constructor node C; port i wired to `⟦eᵢ⟧` |
| `let x :_q τ = e₁ in e₂` | Translate as `(λ(x :_q τ). ⟦e₂⟧) ⟦e₁⟧` |
| `match e { ... }` | Dispatch net (§5.2); ε fires on original node at binding |
| `view e { ... }` | Dispatch net (§5.2); original node wired through |
| `e₁ op₂ e₂` | Primitive node `op₂`; left to `⟦e₁⟧`, right to `⟦e₂⟧` |

> **NOTE on `_ω` and δ-agents**: When translating `λ(x :_ω τ). e`, a single δ-agent is inserted at the var-port. For each use of `x` in `e`, one auxiliary port of the δ-chain is consumed. Multiple uses of `x` create a δ-chain. This is the mechanism by which sharing is realized — the δ-agent handles duplication at runtime, not at the source level. There is no source-level "contract" rule; the type checker merely permits multiple uses of `_ω` bindings, and the translator handles the rest.

### 6.3 Interaction Rules

| Rule | Fires When | Effect |
|------|------------|--------|
| `λ ⋈ @` (β-lin) | Linear λ meets @-node | Substitute body; no δ inserted |
| `λ ⋈ δ` (β-dup) | Shared λ meets δ-node | Duplicate the λ-body graph |
| `λ ⋈ ε` (β-drop) | Any λ meets ε-node | Erase body and var recursively |
| `δ ⋈ δ` | Two δ-agents meet | Commute (rearrange graph) |
| `δ ⋈ ε` | δ meets ε | Erase both branches |

### 6.4 Properties

**Confluence**: Interaction system is confluent (Church-Rosser). Proof: Lafont (1990) — at most one interaction rule per agent pair; no critical pairs.

**Memory Safety**: At termination, no ε-reachable node exists in the normal form. Proof obligation: show every `_1` node has an ε-agent matched to it by the linearity invariant (Theorem 3). Deferred to Phase 0 for `λ◦₀`.

**Optimality** (weakened claim): The translated net shares `_ω` subterms via δ-agents, avoiding re-evaluation of shared subterms. Full Lévy optimality is **not claimed** for the initial implementation. A practical evaluator (sequential, innermost-first) is used in Phase 2. Lévy optimality is deferred to Phase 4.

---

## 7. Concurrency Safety (S5′)

### 7.1 Motivation

V1.0 S5 stated syntactic variable disjointness. This was insufficient: disjoint free variables do not prevent mutable state sharing through aliasing or reference types. S5′ replaces it with a graph-structural invariant.

### 7.2 Revised Property S5′

**Definition (δ-root)**: A δ-agent `d` in a net `N` is a *root δ-agent* of subgraph `G ⊆ N` if:
1. `d`'s principal port is connected to a node **outside** `G`
2. At least one auxiliary port of `d` is connected to a node **inside** `G`

**S5′ (Parallel Isolation)**: If `e₁ ⊗_par e₂` arises from a `(λ ⋈ δ)` interaction (β-dup), then:

```
root-δ-agents(⟦e₁⟧) ∩ root-δ-agents(⟦e₂⟧) = ∅
```

**Why this is sufficient**: δ-agents are the exclusive mechanism by which `_ω` (shared) values are duplicated. If two parallel subgraphs share no root δ-agents, neither can access a shared value also accessible to the other. `_1` values are consumed and not shared; `_&` values are read-only by type. Therefore no mutable state is shared between parallel subgraphs — satisfying R3 (no mutable state sharing in concurrent branches).

**Proof obligation** (Phase 0): Formalize and prove S5′ for `λ◦₀`. The key lemma: after a `(λ ⋈ δ)` interaction, the two resulting subgraphs are structurally disjoint in δ-roots by construction of the translation `⟦·⟧`. This follows from the fact that δ-agents are introduced only at `_ω` binding sites (§6.2), and β-dup copies the body graph without sharing δ-roots from outside.

### 7.3 S5′ Verification Algorithm

S5′ is verified at compile time, **on the translated net before any reduction steps**. The net produced by `⟦·⟧` is acyclic by construction (translation is structurally inductive on terms); this ensures the reachability computation in the algorithm terminates.

```
function verify_S5_prime(net N):
  -- Precondition: N is the output of ⟦·⟧ (acyclic, no reduction steps applied)
  for each δ-agent d in N:
    let G₁ = nodes reachable from d.aux₁ (DFS, no revisiting)
    let G₂ = nodes reachable from d.aux₂ (DFS, no revisiting)
    let roots₁ = { d' ∈ δ-agents(G₁) | d'.principal ∉ G₁ }
    let roots₂ = { d' ∈ δ-agents(G₂) | d'.principal ∉ G₂ }
    if roots₁ ∩ roots₂ ≠ ∅:
      return FAIL("S5′ violation: shared δ-root between parallel subgraphs")
  return PASS
```

**Complexity**: O(|N|) — each node visited at most twice (once per DFS from each auxiliary port of each δ-agent). In practice, DFS uses a visited set to avoid re-traversal.

> **CORRECTION (v2.2)**: Added the precondition that S5′ is checked only on the freshly-translated net (before reduction). The v2.1 algorithm did not state this, making the recursive `root-δ-agents` call potentially non-terminating on cyclic intermediate nets. Since `⟦·⟧` produces acyclic nets and S5′ is a static check, this is the correct framing.

### 7.4 Runtime Parallel Execution

When S5′ holds, subgraphs `⟦e₁⟧` and `⟦e₂⟧` may be reduced in parallel on separate threads with no synchronization required. Implementation uses a **work-stealing thread pool** with per-thread reduction queues.

### 7.5 Immutable ω Values (Known Limitation)

> **NOTE (v2.2)**: S5′ treats all `_ω` values as potentially mutable, because the specification does not distinguish mutable from immutable shared values. **Consequence for users**: programs that use purely functional shared data (immutable `_ω` values) will be serialized unnecessarily by S5′ — the check is sound (no false negatives) but not complete (admits false positives that reject safe parallel execution). This is a known over-conservatism. See §14 for the trade-off. A future `_ωᵢ` (immutable shared) multiplicity is identified as a v2.0 enhancement (§13, Q5).

---

## 8. Trait System

### 8.1 Global Implementation Registry

```
Σ : TraitName × Type → Implementation
```

Built at **link time** (§11). Coherence (R4) requires:

```
∀ C, τ : at most one entry (C, τ) ∈ dom(Σ)
```

Duplicate implementations are a `CoherenceViolation` link-time error.

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
─────────────────────────────────────────
Γ ⊢ Λα. λ(x :_q C α). e : ∀α. C α →^q τ
```

**Trait-App** (instantiation):

```
Γ ⊢ e : ∀α. C α →^q τ     τ' concrete     (C, τ') ∈ Σ
──────────────────────────────────────────────────────────
Γ ⊢ e [τ'] : C τ' →^q τ[τ'/α]
```

### 8.3 Resolution Algorithm

Trait method resolution uses depth-first search over the supertrait DAG:

```
resolve(C, τ, Σ):
  1. If (C, τ) ∈ Σ: return Σ(C, τ)           -- direct impl
  2. For each C' in supertraits(C) (DFS, declaration order):
       if (C', τ) ∈ Σ: return Σ(C', τ)        -- inherited
  3. Error: TraitNotFound
```

**Termination**: Guaranteed (supertrait relation is a DAG, acyclic by declaration rule).  
**Determinism**: Fixed by declaration order in step 2; coherence forbids overlapping implementations.

### 8.4 Trait Inheritance

```
trait C₂ : C₁ where   -- C₂ inherits from C₁
  method2 : τ
```

Implementing `C₂ τ` does **not** automatically satisfy `C₁ τ`. Both must be explicitly implemented. Inherited methods from `C₁` are accessible via resolution when `C₂` is in scope.

Default method implementations are **not supported** in v1.0.

### 8.5 Dispatch

**Static dispatch only** in v1.0. Trait objects (existential `∃τ. C τ` with vtable) are **deferred to v2.0**.

---

## 9. Native Types

### 9.1 Supported Native κ Values (v1.0)

| Kind `κ` | Description | Multiplicity | Representation |
|----------|-------------|--------------|----------------|
| `Int` | 64-bit signed integer | `ω` (copyable) | Register (64-bit) |
| `Float` | 64-bit IEEE 754 float | `ω` (copyable) | Register (64-bit) |
| `Bool` | Boolean | `ω` (copyable) | Register (1-bit logical) |
| `Char` | Unicode scalar value | `ω` (copyable) | Register (32-bit) |
| `Unit` | Single-inhabitant type | `0` (erased) | No representation |

> **CORRECTION (v2.2)**: Unit type description changed from "Zero-information value" to "Single-inhabitant type; no runtime representation." Unit carries the type-level information that it has exactly one inhabitant (`()`). This information is meaningful to the type system (it distinguishes Unit from empty types). At runtime, Unit has no representation and is always erased. The previous description "zero-information" was inconsistent with Unit's role as a type with a unique, constructable inhabitant.

All native types carry multiplicity `ω`: they are copyable by value, stored in registers, and have no heap representation in the interaction net.

### 9.2 Native Operations

```
op₂ ::= + | - | * | / | % | == | != | < | > | <= | >= | && | ||
op₁ ::= - | !
```

Native operations are typed by a separate **primitive type system**:

```
(+) : Int →^ω Int →^ω Int
(-) : Int →^ω Int →^ω Int
(/) : Int →^ω Int →^ω Int     -- division by zero: runtime trap (§10.3)
(==) : Int →^ω Int →^ω Bool
...
```

All argument multiplicities are `ω` (values are copied into the operation).

### 9.3 Hybrid Execution Model

Native values are **opaque leaves** in the interaction net. They do not participate in ε-agent or δ-agent interactions. Arithmetic operations are handled by a separate **primitive reduction system**.

**Interface between the two systems** (reduction interleaving policy):
1. Whenever a primitive redex is available (`e₁ op₂ e₂` where `e₁, e₂` are native literals), it reduces first.
2. After all available primitive redexes are reduced, interaction net rules fire.
3. The two systems are interleaved step-by-step: primitive step, then net steps, then primitive step, etc.

**Formal status**: The primitive reduction system is a separate operational semantics that is **not** part of the interaction net model. It is integrated at the evaluation loop level. Formal properties (preservation, progress) for native operations are stated separately and hold trivially (operations on literals always produce literals; no linearity violations possible since all native types have multiplicity `ω`).

**Exceptional conditions** from native operations (arithmetic overflow, division by zero) are **runtime traps** that terminate the program with a diagnostic. They do not constitute undefined behavior.

---

## 10. Error Model

### 10.1 Compile-Time Errors (All Static)

| Error | Trigger |
|-------|---------|
| `LinearityViolation` | A `_1` variable used ≠ 1 times |
| `BorrowContextMix` | `& + q` for `q ∈ {0,1,ω}` in context composition |
| `OwnershipEscape` | `&` value referenced outside its lexical scope |
| `TraitNotFound` | `C::method(e)` with no `(C, τ) ∈ Σ` |
| `DuplicateImpl` | Two `impl C τ` for same `(C, τ)` within one module |
| `NonExhaustivePattern` | `match`/`view` missing constructor arms |
| `MultiplicityMismatch` | Explicit annotation conflicts with inferred multiplicity |
| `StrictPositivityViolation` | Inductive type with negative type variable occurrence |
| `UndefinedTrait` | `impl C τ` for undeclared trait `C` |
| `BorrowInMatchArm` | Explicit `_&` annotation in a `match` field |

### 10.2 Link-Time Errors

| Error | Trigger |
|-------|---------|
| `CoherenceViolation` | Duplicate `impl C τ` across modules |
| `MissingImpl` | Trait method call resolves to no implementation at link time |
| `S5PrimeViolation` | S5′ check fails on linked net |

### 10.3 Runtime Conditions

Well-typed λ◦ programs have **no undefined behavior**. The following runtime conditions are possible but produce diagnostics, not undefined behavior:

- **Stack overflow** from unbounded recursion: terminates with diagnostic
- **Resource exhaustion** (OOM): terminates with diagnostic
- **Arithmetic trap** (division by zero, integer overflow): terminates with diagnostic

No null pointers. No use-after-free. No data races (by S5′). No uninitialized memory.

---

## 11. Module System & Separate Compilation

### 11.1 Sealed-World Assumption

Coherence is checked at **link time across all modules**. All modules that will ever be linked together must be present at link time.

**Rationale**: Avoids the orphan instance problem. Simpler than per-crate orphan rules (Rust). Consistent with Haskell's approach.

**Known limitation**: Dynamic module loading (plugins) is not supported in v1.0. Users requiring extensibility must wait for v2.0 (`extern` instances or equivalent).

### 11.2 Import/Export

```
module M where
  import M₁           -- import all public names from M₁
  import M₂ (f, g)   -- selective import

export (T, C, f)      -- explicit export list
```

Implementations (`impl`) are **never explicitly exported**. They are globally visible to the linker for coherence checking. Trait implementations are ambient.

### 11.3 Compilation Units

Each source file is one module. Compilation proceeds:

```
Per-module:  parse → type-check → net-translate → emit .λo object
Link:        collect all .λo → coherence check → S5′ check → executable
```

Per-module compilation uses exported type signatures. Coherence and S5′ checks require all modules. Parallel per-module compilation is supported.

---

## 12. Implementation Roadmap

### Phase 0: Formal Kernel (Prerequisite — No Code Until Complete)

Goal: Verified metatheory for `λ◦₀` (multiplicities `{0, 1, ω}`, no traits, no native types, no match/view).

**Deliverables**:
1. Mechanized proofs (Agda or Coq) of Lemma 1 and Theorems 1–4
   - All inductive cases from §4.4 must be discharged
   - Particular attention to App and Let cases (scaling composition)
2. Formal definition of `⟦·⟧` for `λ◦₀`
3. Type preservation proof for translation (Theorem 4)
4. S5′ proof sketch for `λ◦₀`: show that β-dup never introduces shared δ-roots
5. Definition of net typing judgment used in Theorem 4

**Estimated effort**: 2–4 months. Engage a formal methods specialist if available.  
**Gate**: Implementation cannot begin until all five deliverables pass review.

### Phase 1: Core Type System

**Phase 1a** (≈ 1 month): Parser + basic multiplicity checking
- Grammar (§2) → AST
- Multiplicity semiring implementation (§3.2)
- Context splitting and addition

**Phase 1b** (≈ 2 months): Full type system + borrow checker
- Context scaling `q · Γ` (including `0 · Γ` semantics)
- All typing rules (§4.3) including corrected Let
- Borrow checker: lexical scope tracking, `OwnershipEscape` detection
- Exhaustiveness checking for `match`/`view` (§5.3)
- Strict positivity checking (§2.6)
- All static errors (§10.1)

### Phase 2: Interaction Net Runtime

**Phase 2a** (≈ 1–2 months): Sequential evaluator
- Graph representation (port-and-wire model)
- Four agents: λ, @, δ, ε
- Five interaction rules (§6.3)
- Primitive reduction for native operations (§9.3)
- Sequential evaluator (innermost-first)
- ε-agent memory release

**Phase 2b** (≈ 2–3 months): Parallel executor
- S5′ verification pass (§7.3) — applied after translation
- Work-stealing thread pool
- Per-thread reduction queues
- Race-condition testing (property-based)

### Phase 3: Trait System + Modules

(≈ 2 months)
- Global registry `Σ`
- Coherence checking at link time
- DAG-DFS resolution (§8.3)
- Module import/export (§11.2)
- Sealed-world link step

### Phase 4: Optimization & Tooling

(ongoing)
- Reduction trace output (for match/view debugging)
- Net graph visualization (DOT export)
- Source-to-net position mapping (debugger support)
- REPL
- Benchmarking infrastructure
- Lévy optimal evaluation (research phase; not required for correctness)

---

## 13. Open Questions (Residual)

### Deferred to v2.0

**Q1: Trait objects / dynamic dispatch**  
Requires existential types `∃τ. C τ` with vtable layout. Deferred.

**Q2: Default trait method implementations**  
Requires inheritance coherence semantics. Deferred.

**Q3: Optimal Lévy evaluation**  
Practical evaluator used in v1.0. Deferred to Phase 4.

**Q4: Non-lexical lifetimes for `&`**  
Lexical scope used in v1.0 (simpler proof theory). Deferred.

**Q5: Immutable ω distinction (`_ωᵢ` vs `_ωₘ`)**  
S5′ is currently over-conservative for immutable shared values (§7.5). Adding an immutable marker would enable safe parallelism for purely functional shared data. Deferred.

**Q6: Module versioning and `extern` instances**  
Not addressed. Will be needed for package ecosystem. Deferred.

### Resolved

| Question | Resolution | Version |
|----------|------------|---------|
| `& + 1` = ? | Type error — orthogonal axes | v2.0 |
| R3 vs S5 | Replaced by S5′ (graph-structural) | v2.0 |
| Observable match/view | ε-firing site + liveness clause | v2.2 |
| `Γ + q·Δ` definition | Pointwise formula | v2.0 |
| `0·Γ` semantics | `_0`-annotated context (not empty) | v2.2 |
| Trait resolution | DAG-DFS | v2.0 |
| Native κ values | {Int, Float, Bool, Char, Unit} | v2.0 |
| Separate compilation | Sealed-world at link time | v2.0 |
| Error model | All type violations = compile errors | v2.0 |
| Contract rule | Removed from source type system | v2.2 |
| Let rule asymmetry | Corrected: `q · Γ₁` for `e₁` | v2.2 |
| Weaken redundant premise | Removed | v2.2 |
| View field coercion | Made explicit via view-coercion rule | v2.2 |
| View observability vacuity | Liveness clause added | v2.2 |
| S5′ algorithm circularity | Acyclicity precondition stated | v2.2 |
| Unit type description | "Single-inhabitant type" | v2.2 |
| Immutable ω consequence | Documented in §7.5 and §14 | v2.2 |

---

## 14. Trade-offs

| Decision | Choice | Rationale | Known Consequence |
|----------|--------|-----------|-------------------|
| `&` as mode vs. quantity | Mode (orthogonal axis) | Resolves undefined `& + 1`; matches operational semantics | Cannot compose with owned contexts |
| Concurrency guarantee | Graph-structural S5′ | Compile-time verifiable; stronger than syntactic disjointness | Over-conservative for immutable `_ω` values — purely functional shared data is serialized unnecessarily (§7.5) |
| Trait coherence | Sealed-world at link time | Simpler than per-module orphan rules | No dynamic plugin loading in v1.0 |
| Optimal evaluation | Practical evaluator first | Lévy optimality too complex for v1.0 baseline | Shared `_ω` subterms may be re-evaluated in some cases |
| Dynamic dispatch | Static only | Matches uniqueness requirement; simpler | No heterogeneous collections without explicit sum types |
| Borrowed lifetimes | Lexical only | Simpler proof theory; no dataflow analysis | Some safe programs rejected (NLL deferred to Q4) |
| FFI | Deferred to v2.0 | Avoids contaminating net semantics | v1.0 is not interoperable with existing code |
| Default trait methods | Not supported v1.0 | Coherence across defaults is complex | Code reuse requires explicit delegation |
| Memory management for `_ω` | Reference counting | Well-understood; compatible with ε-agents for `_1` | RC overhead on all `_ω` values; no cycle detection |
| Contract rule | Removed from source; handled in translation | Source-level simultaneous substitution is not in the grammar; δ-agents handle sharing transparently | No explicit source-level "make this shared" operation; annotation `_ω` is sufficient |

---

## 15. Bibliography

### Primary References

1. **Atkey, R.** — "Syntax and Semantics of Quantitative Type Theory" (LICS 2018). *Primary grounding for the {0, 1, ω} semiring as a resource semiring.*
2. **Lafont, Y.** — "Interaction Nets" (POPL 1990). *Foundational paper for the execution model.*
3. **Girard, J.-Y.** — "Linear Logic" (Theoretical Computer Science, 1987). *Theoretical basis for linear types.*
4. **Wadler, P.** — "Linear Types Can Change the World!" (IFIP TC 2, 1990). *Practical motivation for linear types.*
5. **Tofte, M. & Talpin, J.-P.** — "Region-Based Memory Management" (Information and Computation, 1997). *Background for lexically-scoped memory (related to `&` borrow semantics).*

### Supporting References

6. **Lévy, J.-J.** — "Optimal Reductions in the Lambda-Calculus" (1980). *Basis for the deferred optimality claim.*
7. **Guzmán, J. & Hudak, P.** — "Single-Threaded Polymorphic Lambda Calculus" (POPL 1990). *Related work on linear polymorphism.*
8. **Honda, K. et al.** — "Multiparty Session Types" (POPL 2008). *Related work on communication safety.*
9. **The Rust Reference** — Ownership and Borrowing. *Engineering reference for `&`-mode semantics.*
10. **Brady, E.** — "Idris 2: Quantitative Type Theory in Practice" (ECOOP 2021). *Production implementation of QTT — closest existing system to λ◦'s type theory.*

---

## 16. Appendix A: Haskell Pseudocode

```haskell
-- Quantity semiring
data Quantity = Zero | One | Omega
  deriving (Eq, Ord, Show)

addQ :: Quantity -> Quantity -> Quantity
addQ Zero q     = q
addQ q    Zero  = q
addQ One  One   = Omega
addQ _    _     = Omega

mulQ :: Quantity -> Quantity -> Quantity
mulQ Zero _    = Zero
mulQ _    Zero = Zero
mulQ One  q    = q
mulQ q    One  = q
mulQ Omega _   = Omega
mulQ _    Omega = Omega

-- Multiplicity (two-axis)
data Multiplicity = Qty Quantity | Borrow
  deriving (Eq, Show)

-- Context composition (fails on mixed Qty/Borrow)
addMul :: Multiplicity -> Multiplicity -> Either TypeError Multiplicity
addMul (Qty q1) (Qty q2) = Right (Qty (addQ q1 q2))
addMul Borrow   Borrow   = Right Borrow
addMul _        _        = Left BorrowContextMix

-- Context scaling (q · Γ)
-- CORRECTION v2.2: 0·Γ produces _0-annotated bindings, not empty context
scaleMul :: Quantity -> Multiplicity -> Either TypeError Multiplicity
scaleMul _ Borrow   = Left BorrowContextMix   -- borrows do not scale
scaleMul q (Qty qv) = Right (Qty (mulQ q qv))

-- Types
data Type
  = TNative NativeKind
  | TArrow Multiplicity Type Type
  | TTrait TraitName Type
  | TInductive TypeName [TypeVar] [Constructor]
  | TForall TypeVar Type
  | TBorrow Type              -- &τ
  | TProduct Type Type        -- (τ₁, τ₂)
  | TSum Type Type            -- τ₁ + τ₂
  deriving (Eq, Show)

data NativeKind = KInt | KFloat | KBool | KChar | KUnit
  deriving (Eq, Show)

-- Contexts
newtype Context = Context (Map Name (Multiplicity, Type))

ctxAdd :: Context -> Context -> Either TypeError Context
ctxAdd (Context m1) (Context m2) =
  fmap Context $ sequenceA $ Map.unionWith combine
    (fmap Right m1) (fmap Right m2)
  where
    combine (Right (mul1, t)) (Right (mul2, _)) =
      fmap (\m -> (m, t)) (addMul mul1 mul2)
    combine r _ = r

ctxScale :: Quantity -> Context -> Either TypeError Context
ctxScale q (Context m) =
  fmap Context $ traverse scaleEntry m
  where
    scaleEntry (mul, t) = fmap (\m -> (m, t)) (scaleMul q mul)

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
  | BorrowInMatchArm Name
  deriving (Show)
```

---

## 17. Appendix B: Correction Log

### v2.2 Corrections (applied in this version)

| ID | Section | Issue | Fix Applied |
|----|---------|-------|-------------|
| C1 | §4.3 | Contract rule present in surface type system (wrong placement; syntax `e[x/x,x/x]` undefined) | Removed Contract rule. Added note: δ-agents are inserted by translator at `_ω` bindings; no source-level rule needed |
| C2 | §4.3 | Let rule scaled only `e₂`'s context, not `e₁`'s — asymmetric for `q = ω` | Corrected: `q · Γ₁ ⊢ e₁` instead of `Γ₁ ⊢ e₁` |
| C3 | §4.3 | Borrow-Weaken rule: identical to Weaken; added noise | Removed |
| C4 | §4.2 | `0·Γ` semantics: empty context vs. `_0`-annotated context unspecified | Defined: `0·Γ` produces `_0`-annotated context; distinct from `∅` |
| C5 | §4.3 | Weaken: redundant premise `x ∉ dom(Γ)` | Removed premise |
| C6 | §5.1 | View field coercion: fields silently overridden to `&` without explanation | Added explicit view-coercion rule and explanation |
| C7 | §5.2 | View observability: `∀ step ε ⋈ C` vacuously true when no ε occurs | Added liveness clause: ∃ `ε ⋈ C` step at or before enclosing scope's ε |
| C8 | §7.3 | S5′ algorithm: recursive `root-δ-agents` call potentially non-terminating on cyclic nets | Added precondition: algorithm runs only on freshly-translated (acyclic) nets |
| C9 | §9.1 | Unit description "Zero-information value" contradicts appendix response | Changed to "Single-inhabitant type; no runtime representation" |
| C10 | §7.5, §14 | Consequence of S5′ over-conservatism for immutable `_ω` not documented | Added §7.5 note and §14 trade-off entry |
| C11 | §4.4 | Phase 0 proof scope: "by induction" without specifying inductive cases | Added explicit inductive cases for Lemma 1 including critical App and Let cases |

### v2.1 Corrections (carried forward)

| ID | Section | Issue | Fix Applied |
|----|---------|-------|-------------|
| B1 | §4.3 | Contract rule direction backwards (ω → 1 instead of 1+1 → ω) | Fixed direction in v2.1 (fully resolved in v2.2 by removal) |
| B2 | §2.3 | Native operations not in grammar | Added `op₂`, `op₁` to term grammar |
| B3 | §2.2 | Product/sum types missing from type grammar | Added `(τ₁, τ₂)` and `τ₁ + τ₂` |
| B4 | §7.3 | S5′ verification algorithm not specified | Added `verify_S5_prime` function |
| B5 | §9.3 | Hybrid execution model undocumented | Added three-step interleaving policy |

---

*Document Version: 2.2*  
*Revised: 2026-03-27*  
*Supersedes: lambda-circle-design-document v2.1*  
*Status: Ready for Phase 0 (Formal Kernel)*
