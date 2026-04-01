# λ◦ (Lambda-Circle) Design Document

**Version**: 2.4
**Date**: 2026-03-29
**Status**: **Phases 0–4 Complete — Module System & Standard Library Specified**
**Confidence**: High (formal proofs complete, module system and stdlib fully designed)

---

## TL;DR

λ◦ is a functional programming language with linear types, traits, and automatic memory
management via Lafont interaction nets. This revision replaces the stub in §11 with a
complete module system specification, adds §16 (Standard Library & Prelude), and
corrects the grammar to match the implemented parser:

- **§2 (Grammar)** — split into concrete surface grammar (§2.A, normative for the
  compiler) and theoretical calculus (§2.B, used in typing rules and Lean4 proofs);
  binary/unary ops removed as syntactic primitives
- **§9 (Native Types)** — operations section removed; ops are prelude trait methods
- **§11 (Module System)** — full specification: one-file-one-module discipline, `pub`
  visibility, export granularity for opaque vs. transparent types, qualified import
  syntax, orphan rule, six-gate compilation pipeline, `.λo` binary format, incremental
  recompilation via export-table hashing, and two-phase link-time verification
- **§16 (Standard Library & Prelude)** — minimal auto-imported prelude (types, traits,
  arithmetic traits, native impls) plus opt-in stdlib modules: `Std.String`,
  `Std.List`, `Std.Map`, `Std.Show`, `Std.IO` — all specified in terms of the
  existing multiplicity system

All content from v2.3 is preserved. New decisions are derived from VT (Value-Transaction)
ontological analysis combined with the Analytical method. Corrections are logged in §19.

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
13. [Open Questions](#13-open-questions)
14. [Trade-offs](#14-trade-offs)
15. [Phase 0: Lean4 Formalization](#15-phase-0-lean4-formalization)
16. [Standard Library & Prelude](#16-standard-library--prelude)
17. [Bibliography](#17-bibliography)
18. [Appendix A: Haskell Pseudocode](#18-appendix-a-haskell-pseudocode)
19. [Appendix B: Correction Log](#19-appendix-b-correction-log)

---

## 1. Problem Statement & Constraints

### Goals

Design and implement λ◦, a functional programming language that statically guarantees:

- **Type safety** — well-typed programs do not go wrong
- **Memory safety** — no use-after-free, no leaks, without garbage collection
- **Concurrency safety** — parallel subgraphs share no mutable state (by construction,
  not by runtime check)
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

> **NOTE FOR IMPLEMENTERS**: This section is normative and is the authoritative reference
> for the λ◦ syntax. It is divided into two subsections:
>
> - **§2.A — Concrete Surface Grammar**: the grammar the parser implements. This is
>   what users write. All examples in this document use this syntax.
> - **§2.B — Theoretical Calculus**: the abstract grammar used in the type rules (§4),
>   net translation (§6), and Lean4 formalization (§15). It extends the surface grammar
>   with forms that exist only in the metatheory, not in user-written code.
>
> When the two grammars diverge, the surface grammar governs what the compiler accepts.
> The theoretical calculus governs what the proofs reason about.

### 2.A Concrete Surface Grammar

This is the grammar the parser accepts. It is the ground truth for all examples.

#### 2.A.1 Terms

```
term      ::= app_expr

app_expr  ::= atom_expr { atom_expr }          -- left-associative juxtaposition

atom_expr ::=
    | "λ" var ":" multiplicity ":" type "." term        -- lambda abstraction
    | "let" var ":" multiplicity ":" type "="
          term "in" term                                 -- let binding
    | "match" term "with" "{"
          pattern "=>" term
          { "|" pattern "=>" term } "}"                  -- match (ownership transfer)
    | "view" term "with" "{"
          pattern "=>" term
          { "|" pattern "=>" term } "}"                  -- view (borrow observation)
    | integer                                            -- Int literal
    | float                                              -- Float literal
    | "true" | "false"                                   -- Bool literals
    | "Unit"                                             -- Unit literal
    | identifier                                         -- variable or constructor
    | "(" term ")"                                       -- grouping
```

Binary and unary operations (arithmetic, comparison, boolean) are **not** syntactic
primitives. They are ordinary functions defined as trait method implementations in
`Std.Prelude` (see §16.2) and applied via juxtaposition like any other function.

#### 2.A.2 Multiplicities

```
multiplicity ::=
    | "0"          -- zero: erased, compile-time only
    | "1"          -- one: linear, used exactly once
    | "ω"          -- omega: unrestricted, used many times
    | "&"          -- borrow: read-only observer, lexically scoped
```

`&` is a **mode annotation**, not a quantity. It cannot appear as the result of context
composition (see §3). Mixing owned-quantity multiplicities (`0`, `1`, `ω`) with `&` in
the same composition is a **static type error**.

#### 2.A.3 Types

The type grammar is the core gap between the current parser and the full language. The
productions are listed below with their implementation status. Productions marked ⚠️ are
**Phase 5a prerequisites** — the module system and stdlib cannot be expressed without
them.

```
-- Top level: function arrow (right-associative)
type      ::= type_app                                   -- ✅ implemented (atom only)
            | type_app "->" type                         -- ⚠️ Phase 5a

-- Type application: juxtaposition, left-associative
-- mirrors app_expr at the term level
type_app  ::= type_atom { type_atom }                   -- ⚠️ Phase 5a

-- Atoms
type_atom ::=
    | "Unit"                                             -- ✅ implemented
    | "Int"                                              -- ✅ implemented
    | "Float"                                            -- ✅ implemented
    | "Bool"                                             -- ✅ implemented
    | "Char"                                             -- ✅ implemented
    | upper_identifier                                   -- ⚠️ Phase 5a (type constructor)
    | lower_identifier                                   -- ⚠️ Phase 5a (type variable)
    | "&" type_atom                                      -- ⚠️ Phase 5a (borrow reference)
    | "(" type ")"                                       -- ✅ implemented
    | "(" type "," type ")"                              -- ⚠️ Phase 5a (product type)
```

**Naming convention** (functional language standard):
- `lower_identifier` — begins with a lowercase letter or `_`: type variables (`a`, `b`,
  `α`, `result`)
- `upper_identifier` — begins with an uppercase letter: type constructors (`Option`,
  `List`, `Bool`, `Map`)

This convention is **enforced by the parser**, not just by style. An uppercase identifier
in type position is always a constructor; a lowercase identifier is always a variable.
This eliminates the ambiguity between `Option` (concrete type) and `a` (universally
quantified variable) without requiring explicit `∀` syntax in the surface grammar.

**Multiplicity on arrows.** The surface grammar uses bare `->` which the type checker
interprets as `→^1` (linear) by default. This covers all stdlib signatures. Explicit
multiplicity annotation on arrows (`→^ω`, `→^&`) remains in §2.B (theoretical calculus)
only — it is not part of the surface syntax in v1.0. Multiplicity is declared on the
**binder** (`λx:ω:τ`), not on the arrow.

**`&` in type position** is a type prefix meaning "borrowed reference to", distinct from
`&` as a multiplicity annotation in binder position. The parser distinguishes them by
context: `&` followed by a type atom is always a reference type; `&` between `:` delimiters
is always a multiplicity.

#### 2.A.4 Patterns

```
pattern   ::= "_"                              -- wildcard (binds with multiplicity 0)
            | identifier                       -- variable (binds with multiplicity 1)
```

Constructor patterns (`C p₁ ... pₙ`) are reserved for a future version. Currently,
constructor scrutiny is handled by matching on an identifier bound to the whole value
and then using `view` for field access.

#### 2.A.5 Module-Level Declarations

All module-level declarations are **Phase 5a/5 prerequisites** — none are parsed by the
current v1.0 parser. They are specified here as the normative target for implementation.

```
decl ::=
    | "pub"? "type" upper_identifier type_param* "=" type      -- type definition
    | "pub"? "type" upper_identifier type_param* "(..)"        -- transparent export
    | "pub"? "val"  lower_identifier ":" type "=" term         -- value binding
    | "pub"? "trait" upper_identifier type_param*
          ( "where" upper_identifier type_param* )?            -- optional supertrait
          "{" sig* "}"                                          -- trait definition
    | "impl" upper_identifier "for" type
          ( "where" constraint* )?
          "{" val_def* "}"                                      -- trait implementation
    | "use" module_path                                         -- qualified import
    | "use" module_path "(" lower_identifier* ")"              -- selective unqualified
    | "use" module_path "(..)"                                 -- full unqualified
    | "use" module_path "as" upper_identifier                  -- aliased import
    | "no_prelude"                                              -- suppress Std.Prelude

-- Trait method signature (inside trait body)
sig     ::= "val" lower_identifier ":" type

-- Trait method implementation (inside impl body)
val_def ::= "val" lower_identifier ":" type "=" term

-- Type constraint (inside where clause)
constraint ::= upper_identifier type

-- Naming
type_param  ::= lower_identifier
module_path ::= upper_identifier { "." upper_identifier }
```

**`impl C for T` vs `impl T : C`.** The surface grammar uses `impl Eq for Int` (trait
first, type second), consistent with functional language conventions (Haskell, OCaml).
The earlier stub used `impl T : C`; this is corrected here. The orphan rule (§8.4)
applies: the impl must live in the module defining `Eq` or the module defining `Int`.

**Constructor patterns in match** are a Phase 5a extension. The current parser handles
only `_` and `lower_identifier` in pattern position. Constructor patterns
(`upper_identifier pattern*`) require the type grammar extensions to be in place first,
since the checker needs type information to validate exhaustiveness.

#### 2.A.6 Concrete Examples

**Terms (✅ implemented):**

```
42                                    -- Int literal
3.14                                  -- Float literal
true                                  -- Bool literal
Unit                                  -- Unit literal
λx:1:Int.x                           -- identity function (linear)
λx:ω:Int.x                           -- identity function (shared)
λx:&:Int.x                           -- identity function (borrow)
(λx:1:Int.x) 5                       -- application
let x:1:Int = 5 in x                 -- let binding
match x with { _ => x }              -- match (wildcard arm)
match x with { _ => x | y => y }     -- match (two arms)
add x y                              -- binary op as function application
neg x                                -- unary op as function application
```

**Types (⚠️ Phase 5a — not yet parsed):**

```
Int -> Bool                          -- function type (arrow)
Int -> Int -> Int                    -- curried binary function
&Int -> Bool                         -- borrow reference in type position
Option Int                           -- type application
Result Int DivisionByZero            -- type application (two args)
List (Option Int)                    -- nested type application
(Int, Bool)                          -- product type
```

**Module declarations (⚠️ Phase 5a/5 — not yet parsed):**

```
pub val identity : Int -> Int = λx:1:Int.x
pub type Option a (..)
impl Eq for Int { val eq = λx:&:Int. λy:&:Int. prim_eq x y }
use Std.List
use Std.List (map, filter)
use Std.List as L
```

---

### 2.B Theoretical Calculus

Used in typing rules (§4), net translation (§6), and Lean4 proofs (§15). Not parsed
by the compiler.

#### 2.B.1 Types (extended)

```
τ ::= Native κ             -- native base type
    | τ →^q τ              -- function type with multiplicity annotation
    | ∀α. τ                -- universally quantified
    | C τ                  -- trait constraint
    | μα. τ                -- inductive type (strictly positive)
    | &τ                   -- borrowed reference (lifetime-erased)
    | (τ₁, τ₂)             -- product type
    | τ₁ + τ₂              -- sum type
```

The surface `->` corresponds to `→^1` in the theoretical calculus when no multiplicity
is written. Explicit multiplicity annotations (`→^q`) appear only in typing rules and
Lean4 code.

#### 2.B.2 Terms (extended)

```
e ::= x                           -- variable
    | λx:q:τ. e                  -- abstraction (q-annotated)
    | e₁ e₂                      -- application
    | let x:q:τ = e₁ in e₂       -- let binding
    | match e with { p → e }⁺    -- match (ownership transfer)
    | view e with { p → e }⁺     -- view (borrow observation)
    | C e₁ ... eₙ                -- constructor application (metatheory only)
    | lit                        -- literal
```

#### 2.B.3 Patterns (extended)

```
p ::= _              -- wildcard (binds with multiplicity 0)
    | x              -- variable (binds with multiplicity 1)
    | C p₁ ... pₙ    -- constructor pattern (metatheory only)
```

---

## 3. Multiplicity System

### 3.1 Quantity Semiring (Q, +, ·, 0, 1)

The quantity axis forms a commutative semiring:

| `+` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 1 | ω |
| **1** | 1 | ω | ω |
| **ω** | ω | ω | ω |

| `·` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 0 | 0 |
| **1** | 0 | 1 | ω |
| **ω** | 0 | ω | ω |

### 3.2 Mode Axis (&)

`&` (borrow) is a **mode**, not a quantity:
- `&` cannot be scaled: `q · &` is undefined
- `&` cannot be added: `& + q` is a static error
- `&` is only used for observation, never consumption

### 3.3 Context Operations

**Addition (splitting)**: `Γ₁ + Γ₂` combines two contexts by adding multiplicities
pointwise.

**Scaling**: `q · Γ` scales all entries in Γ by q.

**Critical**: `0 · Γ` produces a context where **every binding is annotated with `_0`**,
NOT the empty context. This is essential for:
- Weakening: `_0` bindings trigger the Weaken rule
- Net translation: `_0` bindings insert ε-agents

---

## 4. Type System

### 4.1 Typing Judgment

```
Γ ⊢ e : τ
```

where `Γ` is a context mapping variables to (multiplicity, type) pairs.

### 4.2 Typing Rules

| Rule | Premise | Conclusion |
|------|---------|------------|
| **Var** | `x :_1 τ ∈ Γ` | `Γ ⊢ x : τ` |
| **Var-Ω** | `x :_ω τ ∈ Γ` | `Γ ⊢ x : τ` |
| **Var-Borrow** | `x :_& τ ∈ Γ` | `Γ ⊢ x : &τ` |
| **Abs** | `Γ, x :_q τ₁ ⊢ e : τ₂` | `Γ ⊢ λx:q:τ₁. e : τ₁ →^q τ₂` |
| **App** | `Γ₁ ⊢ e₁ : τ₂ →^q τ` , `Γ₂ ⊢ e₂ : τ₂` | `Γ₁ + q·Γ₂ ⊢ e₁ e₂ : τ` |
| **Let** | `Γ₁ ⊢ e₁ : τ₁` , `Γ₂, x:_q τ₁ ⊢ e₂ : τ₂` | `Γ₁ + q·Γ₂ ⊢ let x:q:τ₁ = e₁ in e₂ : τ₂` |
| **Weaken** | `Γ ⊢ e : τ` | `Γ, x:_0 τ' ⊢ e : τ` |

---

## 5. Match / View Semantics

### 5.1 Match (Ownership Transfer)

- Fields are bound with multiplicity **1** (ownership transfer)
- Cannot observe borrowed data

### 5.2 View (Borrow Observation)

- Fields are coerced to `&` (view-coercion rule)
- Requires **observability liveness**: at least one ε-agent interacts with the view
  before the enclosing scope ends

---

## 6. Interaction Net Translation

### 6.1 Agents

The agent set is extended in v2.4 with two primitive agent classes to support the
evaluation of built-in operations without a hybrid evaluator.

| Agent | Arity | Purpose |
|-------|-------|---------|
| `λ` | 3-port | Lambda abstraction |
| `@` | 3-port | Application |
| `δ` | 3-port | Duplication (for `ω`) |
| `ε` | 1-port | Erasure (for `0` and scope-end of `1`) |
| `Prim(op)` | 2-port (unary) or 3-port (binary) | Primitive operation — compiler-internal only |
| `PrimVal(type, value)` | 1-port | Typed primitive value (Int, Float, Bool, Char) |
| `PrimIO(op)` | 3-port | IO primitive operation — compiler-internal only |
| `IO_token` | 1-port | Linear sequencing token — threads through IO actions |

**`PrimIO(op)` is 3-port**: principal port + `IO_token` port + argument port. It
consumes the incoming `IO_token`, performs the side effect, and produces a fresh
`IO_token` paired with the result. This port structure enforces sequencing: the next
IO action cannot fire until the previous one has produced a fresh token.

**`IO_token` is linear (`_1`).** It cannot be duplicated (δ-agent is forbidden on it)
or silently erased (ε-agent is forbidden on it). The only agents that consume an
`IO_token` are `PrimIO` agents. This guarantees IO actions are executed exactly once
and in the order dictated by the `bind` chain. The token is created by the runtime at
program entry and discarded only when `main` returns. They are emitted exclusively by
the net translator when it encounters a `prim_*` intrinsic call (§6.4). No λ◦ source
program can reference them directly. User code reaches primitive operations only through
trait method wrappers defined in `Std.Prelude` (§16.2).

**`PrimVal(type, value)` carries a typed payload:**
- `PrimVal(Int, 42)`
- `PrimVal(Float, 3.14)`
- `PrimVal(Bool, true)`
- `PrimVal(Char, 'a')`

The type tag is redundant for correct programs (the type checker already guarantees
well-typedness) but is retained for the trace debugger and for catching translator bugs
at the agent boundary.

**Arity by operation kind:**
- Binary operations (`prim_iadd`, `prim_feq`, etc.) → `Prim(op)` is **3-port**:
  principal port + two auxiliary ports for the two operands
- Unary operations (`prim_ineg`, `prim_not`, etc.) → `Prim(op)` is **2-port**:
  principal port + one auxiliary port

This matches hardware arithmetic structure and avoids the overhead of two interaction
rule firings per binary operation that a uniformly-curried 2-port design would require.

### 6.2 Interaction Rules

**Structural rules** (unchanged from v2.3):

| Rule | Fires When | Effect |
|------|------------|--------|
| `λ ⋈ @` | Linear λ meets @ | β-reduction: substitute body |
| `λ ⋈ δ` | Shared λ meets δ | Duplicate body graph |
| `λ ⋈ ε` | Any λ meets ε | Erase body |
| `δ ⋈ δ` | Two δs meet | Commute |
| `δ ⋈ ε` | δ meets ε | Erase both branches |

**Primitive rules** (new in v2.4):

| Rule | Fires When | Effect |
|------|------------|--------|
| `Prim(bin_op) ⋈ PrimVal(t, v₁) ⋈ PrimVal(t, v₂)` | Binary op meets two typed values | Compute result → `PrimVal(t, result)` |
| `Prim(un_op) ⋈ PrimVal(t, v)` | Unary op meets typed value | Compute result → `PrimVal(t', result)` |
| `PrimVal ⋈ ε` | Any PrimVal meets ε | Erase value — no hook needed |
| `PrimVal ⋈ δ` | Any PrimVal meets δ | Duplicate value — `Copy` is always satisfied for primitives |

**IO rules** (new in v2.4):

| Rule | Fires When | Effect |
|------|------------|--------|
| `PrimIO(op) ⋈ IO_token ⋈ arg` | IO op meets token and argument | Execute side effect; produce `(IO_token_new, result)` |
| `PrimIO(op) ⋈ IO_token` | IO op with no argument (e.g. `read_line`) | Execute side effect; produce `(IO_token_new, result)` |

**`IO_token` cannot interact with `ε` or `δ`.** Attempting to erase or duplicate an
`IO_token` is a `LinearityViolation` caught at Gate 3. This is enforced by the type
system: `IO_token` does not implement `Drop` or `Clone`.

The evaluator loop is **uniform**: it scans for all active pairs (principal-port
connections between any two agents) and fires the appropriate rule. There is no
separate "primitive evaluation mode" — `Prim ⋈ PrimVal` is handled identically to
`λ ⋈ @`. This eliminates the hybrid evaluator from the implementation entirely.

**Type mismatch at a primitive rule** (e.g. `Prim(IAdd) ⋈ PrimVal(Bool, true)`) is
a translator bug, not a runtime error. The type checker guarantees this cannot happen
in a well-typed program. The evaluator may assert-fail in debug builds.

### 6.3 Translation Policy

- `_1` bindings → direct connection
- `_ω` bindings → insert δ-agent
- `_0` bindings → insert ε-agent
- `_&` bindings → observe-only (no agent)

### 6.4 Primitive Translation

When the net translator encounters a `prim_*` intrinsic applied to arguments, it emits
`Prim` and `PrimVal` agents directly rather than going through the λ/@ encoding. This
is the only point where the translator deviates from the standard term-to-net mapping.

**Translation of a literal:**
```
translate(42 : Int)  →  PrimVal(Int, 42)
translate(true)      →  PrimVal(Bool, true)
```

**Translation of a primitive application:**
```
translate(prim_iadd x y)
  →  Prim(IAdd) with aux1 wired to translate(x)
                     aux2 wired to translate(y)
```

**Translation of a prelude wrapper** (how user code reaches primitives):
```
-- Prelude source:
val add : Int -> Int -> Int = λx:1:Int. λy:1:Int. prim_iadd x y

-- Net translation: the λ/@ encoding wraps the Prim(IAdd) node.
-- After β-reduction of (add 3 5), the net contains:
--   Prim(IAdd) ⋈ PrimVal(Int, 3) ⋈ PrimVal(Int, 5)
-- which fires immediately to:
--   PrimVal(Int, 8)
```

The wrapper cost (two β-reductions to unwrap `add`) is paid once per call site. The
actual primitive computation fires in a single interaction rule. For `ω`-shared
functions the δ-agent duplicates the wrapper graph, not the result.

**Translation of IO intrinsic calls** differs from arithmetic in one critical way: the
translator maintains a **current `IO_token` wire** as part of its state. Each
`prim_io_*` call consumes the current token wire and produces a fresh one, which
becomes the new current token for the next call.

```
-- Translation state carries: current_token_wire

translate(prim_io_println s, state):
  node ← PrimIO(Println) with:
    token_in  ← state.current_token_wire   -- consume current token
    aux       ← translate(s)
    principal → (token_out, result_wire)   -- produce fresh token + result
  state.current_token_wire ← token_out    -- advance token for next IO call
  return result_wire                       -- IO Unit result

translate(prim_io_read_line, state):
  node ← PrimIO(ReadLine) with:
    token_in  ← state.current_token_wire
    principal → (token_out, result_wire)   -- IO (Result String IOError)
  state.current_token_wire ← token_out
  return result_wire

translate(prim_io_close f, state):
  node ← PrimIO(Close) with:
    token_in  ← state.current_token_wire
    aux       ← translate(f)              -- File value, consumed linearly
    principal → (token_out, result_wire)
  state.current_token_wire ← token_out
  return result_wire
```

**`bind` in the `Monad IO` instance is the thread.** The `bind` implementation for
`IO` desugars to sequential token threading: the token output of the first action
becomes the token input of the continuation. The translator sees `bind` chains as
sequences of `prim_io_*` calls with the token wire connecting them automatically.

```
-- bind (println "a") (λ_. println "b") translates to:

PrimIO(Println)[token=T0, aux=PrimVal(String,"a")] → (T1, Unit)
PrimIO(Println)[token=T1, aux=PrimVal(String,"b")] → (T2, Unit)

-- T0 is the runtime-provided initial token
-- T1 gates the second println — it cannot fire until the first completes
-- T2 is the final token returned by main
```

---

## 7. Concurrency Safety (S5′)

### 7.1 Definition

A net is **S5′-safe** if for every δ-agent, the two subgraphs below its auxiliary ports
share no δ-agents in common.

### 7.2 Theorem

S5'-safe nets can be evaluated in parallel without data races.

### 7.3 Verification Algorithm

```
verify_S5'(net):
  for each δ-agent d in net:
    let G1 = nodes reachable from d.aux1
    let G2 = nodes reachable from d.aux2
    let roots1 = root_δ_agents(G1)
    let roots2 = root_δ_agents(G2)
    if roots1 ∩ roots2 ≠ ∅:
      return Violation
  return Safe
```

**Precondition**: Algorithm runs only on freshly-translated (acyclic) nets.

### 7.4 Two-Phase Verification

S5′ is verified at two points in the pipeline:

1. **Per-module** (Gate 5, §11.2): each module's net is verified in isolation after
   translation
2. **Global** (Link Step L3, §11.4): the fully composed net is verified after linking,
   because cross-module δ-agent interactions that are individually safe may become unsafe
   when composed

Both checks are necessary. Per-module verification catches local violations early.
Global verification is the soundness guarantee.

### 7.5 Conservative Over-approximation

The S5′ check may reject some nets that are safe in practice. This is an intentional
conservative trade-off: soundness is preferred over completeness for the parallelism
guarantee. Rejected programs can be restructured to make isolation explicit.

---

## 8. Trait System

### 8.1 Global Registry

```
Σ : (TraitName × Type) → Implementation
```

### 8.2 Coherence

At link time: ensure at most one `impl C τ` per type-trait pair.

Error: `CoherenceViolation(trait, type, module_a, module_b)` — reports both conflicting
modules.

### 8.3 Resolution

DAG-DFS algorithm with memoization for deterministic method resolution:

```
resolve(trait_name, ty, registry):
  if (trait_name, ty) ∈ registry:
    return registry[(trait_name, ty)]      -- direct impl
  for supertrait in supertraits(trait_name):
    if (supertrait, ty) ∈ registry:
      return registry[(supertrait, ty)]    -- inherited
  raise TraitNotFound
```

### 8.4 Orphan Rule

An `impl C τ` is only legal in:
- The module that **defines** `C`, or
- The module that **defines** `τ`

Any other module attempting an impl produces a compile-time `OrphanImpl` error. This
rule makes global coherence decidable without whole-program analysis and is enforced
per-module at Gate 3 (§11.2), before link time.

### 8.5 Impl Visibility

`impl` blocks are **never re-exported**. Each impl is owned by exactly one module — its
defining module. The coherence checker attributes impls to their defining module, not to
any re-exporter. When a type or trait is imported, all impls connecting them become
visible to the type checker automatically, without an explicit import statement.

---

## 9. Native Types

### 9.1 Type Definitions

| Type | Inhabitants | Representation |
|------|-------------|----------------|
| `Int` | ℤ | 64-bit two's complement |
| `Float` | ℝ (IEEE 754) | 64-bit double precision |
| `Bool` | `{true, false}` | 1 bit (byte-aligned) |
| `Char` | Unicode scalar values | 32-bit |
| `Unit` | `{}` | 0 bits (zero-width) |

All five native types are available in every module without an explicit import. They are
defined in the compiler, not in any `.λ` source file.

### 9.2 Operations

Native type operations — arithmetic, comparison, boolean logic — are **not** syntactic
primitives or built-in special forms. They are ordinary functions defined as trait method
implementations in `Std.Prelude` (§16.2) and applied like any other function:

```
add x y      -- not: x + y
neg x        -- not: -x
eq x y       -- not: x == y
```

At the **net level**, each trait method wrapper delegates to a `prim_*` compiler
intrinsic (§16.3), which the net translator converts into a `Prim(op)` agent (§6.1).
The evaluator reduces `Prim(op) ⋈ PrimVal` pairs by the primitive interaction rules
(§6.2) — no hybrid evaluation mode is needed. From the user's perspective every
operation is a named function; from the evaluator's perspective every operation is an
interaction rule.

### 9.3 Notes

**`Float` and `Eq`**: IEEE 754 defines `NaN ≠ NaN`, which violates reflexivity of `Eq`.
`Float` implements `Eq` with this known caveat, documented here as a deliberate
exception. As a consequence, `Float` does **not** implement `Hash` — the `Hash` contract
requires `a == b → hash(a) == hash(b)`, which NaN breaks.

**`Unit`**: zero-width type used as the return type of effectful functions that produce
no value. The ε-agent erases `Unit` values at no cost.

---

## 10. Error Model

All errors are **static compile-time errors**:

| Error | Phase | Description |
|-------|-------|-------------|
| `ParseError` | Gate 1 | Malformed token stream or syntax |
| `ModuleNotFound` | Gate 2 | Imported module does not exist |
| `NameNotFound` | Gate 2 | Imported name not exported by module |
| `CycleDetected` | Gate 2 | Cyclic dependency in import graph |
| `OrphanImpl` | Gate 3 | `impl` not in defining module of trait or type |
| `LinearityViolation` | Gate 3 | Variable used more/less than its multiplicity allows |
| `BorrowContextMix` | Gate 3 | Borrow mode mixed with quantities in context composition |
| `MultiplicityMismatch` | Gate 3 | Declared and inferred multiplicities differ |
| `TraitNotFound` | Gate 3 | No implementation found for trait-type pair |
| `DuplicateImpl` | Gate 3 | Multiple impls for same trait-type pair in one module |
| `StrictPositivityViolation` | Gate 3 | Inductive type parameter appears to left of `→` |
| `OwnershipEscape` | Gate 4 | Borrowed reference escapes its lexical scope |
| `NonExhaustivePattern` | Gate 4 | Match does not cover all constructors |
| `BorrowInMatchArm` | Gate 4 | Borrow mode used inside match arm |
| `S5PrimeViolation` | Gate 5 / L3 | Net fails S5′ safety check |
| `UnknownPrimitive` | Gate 5 | `prim_*` name not in the closed intrinsics table (§16.3) |
| `CoherenceViolation` | L2 | Two modules define impl for same (trait, type) pair |
| `SerializationError` | Gate 6 | `.λo` serialization failure |

---

## 11. Module System & Separate Compilation

> **This section replaces the stub from v2.3 with a complete specification.**

### 11.1 Module Unit

**One file, one module.** Each `.λ` source file defines exactly one module. The module
name is derived from the file path relative to the project root, with `/` replaced by
`.` and the `.λ` extension stripped:

```
src/Data/List.λ   →  module name: Data.List
src/Main.λ        →  module name: Main
```

The `module` keyword is reserved for future nested module support (v2.0) but has no
semantics in v1.0. The module name must match the file path — a mismatch is a
`ModuleNameMismatch` error at Gate 1.

### 11.2 Visibility & Export

**Private by default.** All declarations are private unless annotated with `pub`. An
accidental export cannot cause a coherence violation in an unsuspecting downstream
module.

Export granularity for type declarations:

| Syntax | Effect |
|--------|--------|
| `pub type T α*` | Exports type name only — **opaque type**. Callers can hold `T α` values but cannot construct or inspect them. |
| `pub type T α* (..)` | Exports type name **and all constructors** — **transparent type**. Callers can pattern-match freely. |
| `pub val x : τ = e` | Exports term binding `x` at type `τ` |
| `pub trait C α where ...` | Exports trait name and all method signatures |
| `impl C τ where ...` | Never exported explicitly — impls are implicit (see §8.5) |

**Re-exports**: types and terms may be re-exported with `pub use Path.Name`. `impl`
blocks are never re-exported.

### 11.3 Import Forms

The module graph must be a **DAG**. Cyclic imports are detected at Gate 2 before
typechecking and reported as `CycleDetected` with all modules in the cycle listed.

Three import forms, in order of preference:

```
-- Qualified import (preferred): access as List.map, List.filter
use Std.List

-- Selective unqualified: brings only named items into scope
use Std.List (map, filter)

-- Full unqualified (discouraged): all exports enter scope directly
use Std.List (..)
```

The default qualifier for `use Std.List` is the last path segment: `List`. The qualifier
can be overridden with `as`:

```
use Std.Collections.PersistentHashMap as Map
```

**Impl visibility on import**: when a type or trait is imported by any form, all `impl`
blocks connecting imported types and traits become visible to the type checker
automatically. No explicit import of impls is required or supported.

**Prelude suppression**: the compiler implicitly prepends `use Std.Prelude (..)` to
every module before name resolution. A module may suppress this with a top-level
`no_prelude` annotation, after which all names — including `Bool`, `Option`, and the
core traits — must be imported explicitly.

### 11.4 Compilation Pipeline

Each `.λ` file is compiled independently through six sequential gates. Each gate is a
transaction: it either fulfills its contract and passes the artifact forward, or it
fails with a typed error and halts. No gate begins until the previous one completes
successfully.

```
.λ source
   │
   ▼ Gate 1 — Lexer / Parser
   │  Input:  .λ text
   │  Output: AST
   │  Errors: ParseError, ModuleNameMismatch
   │
   ▼ Gate 2 — Name Resolution
   │  Input:  AST + import graph (DAG)
   │  Output: Resolved AST (all names bound to their defining modules)
   │  Errors: ModuleNotFound, NameNotFound, CycleDetected
   │
   ▼ Gate 3 — Type Checker + Multiplicity + Orphan Check
   │  Input:  Resolved AST
   │  Output: Typed AST
   │  Errors: LinearityViolation, BorrowContextMix, MultiplicityMismatch,
   │          TraitNotFound, DuplicateImpl, StrictPositivityViolation, OrphanImpl
   │
   ▼ Gate 4 — Borrow Checker + Exhaustiveness
   │  Input:  Typed AST
   │  Output: Verified Typed AST
   │  Errors: OwnershipEscape, NonExhaustivePattern, BorrowInMatchArm
   │
   ▼ Gate 5 — Net Translation + Per-module S5′ Verification
   │  Input:  Verified Typed AST
   │  Output: Interaction net (S5′-verified in isolation)
   │  Errors: S5PrimeViolation
   │
   ▼ Gate 6 — Serialization
      Input:  Verified net + type info + export table
      Output: .λo binary
      Errors: SerializationError
```

### 11.5 The `.λo` Object Format

`.λo` is a **MessagePack binary** with five named sections. Downstream consumers read
`.λo` files without access to source.

| Section | Contents |
|---------|----------|
| **Type section** | All exported types, their kinds, constructors, and multiplicities |
| **Trait section** | All exported traits and all `impl` blocks defined in this module |
| **Net section** | Full compiled interaction net for each exported term (v1.0: full nets; v2.0: templated nets with named ports) |
| **Export table** | Map from public names to locations in the above sections |
| **Debug section** | Source positions, original names before qualification (for trace debugger and error messages) |

Each `.λo` carries a **content hash of its export table**. The build system compares
hashes before deciding whether downstream modules need recompilation. If the net section
changes but the type section does not (e.g. an internal optimization), dependents do not
recompile — only the linker reruns.

**v1.0 note on net section**: nets are stored complete and self-contained (no holes).
This means shared dependencies may be duplicated across `.λo` files. Templated nets with
named ports — eliminating duplication at the cost of a more complex linker — are
deferred to v2.0.

### 11.6 Link Step

The linker runs after all `.λo` files exist. It executes four steps in order:

```
All .λo files
   │
   ▼ Step L1 — Build global registry Σ
   │  Collect all trait sections across all .λo files
   │  Produce: Σ : (TraitName × Type) → (Implementation, DefiningModule)
   │
   ▼ Step L2 — Global coherence check
   │  Assert: no (TraitName × Type) pair has more than one impl across all modules
   │  Error:  CoherenceViolation(trait, type, module_a, module_b)
   │
   ▼ Step L3 — Global S5′ verification
   │  Compose all nets into one graph
   │  Run verify_S5′ on the composed net
   │  Error:  S5PrimeViolation(location)
   │
   ▼ Step L4 — Emit executable
      Wire all nets together
      Emit custom ELF-like binary
```

**Why two S5′ passes?** Gate 5 catches violations within a single module. Step L3 catches
violations that only become apparent when cross-module nets are composed — a δ-agent in
module A and a δ-agent in module B may share a subgraph that neither could see in
isolation.

---

## 12. Implementation Roadmap

| Phase | Description | Duration | Status |
|-------|-------------|----------|--------|
| 0 | Lean4 Formal Kernel | 16 weeks | ✅ Complete |
| 1 | Core Type System (Rust) | 3 months | ✅ Complete |
| 2 | Interaction Net Runtime | 5 months | ✅ Complete |
| 3 | Trait System + Modules | 2 months | ✅ Complete |
| 4 | Tooling | Ongoing | ✅ Complete |
| 5a | Grammar Extensions (prerequisite) | 2 weeks | ⏳ Next |
| 5 | Module System (v2.4 spec) | 6 weeks | ⏳ Blocked on 5a |
| 6 | Standard Library (v2.4 spec) | 8 weeks | ⏳ Blocked on 5 |

**Phase 5a — Grammar Extensions** is a hard prerequisite for everything that follows.
Without it, no module declaration, no trait definition, and no stdlib signature can be
parsed. It covers exactly the five gaps identified in the grammar analysis:

| Extension | Production | Unblocks |
|-----------|------------|---------|
| Naming convention | `upper_identifier`, `lower_identifier` | Type variables vs constructors |
| Type variables | `lower_identifier` in `type_atom` | `Option a`, `List a`, `Result a e` |
| Type application | `type_app ::= type_atom { type_atom }` | `Option Int`, `List Char` |
| Reference in type position | `"&" type_atom` in `type_atom` | `&Int -> Bool`, trait signatures |
| Arrow types | `type_app "->" type` in `type` | All function signatures |
| `impl C for T` syntax | See §2.A.5 | Trait implementations |
| Constructor patterns | `upper_identifier pattern*` in `pattern` | Exhaustiveness checking |

**Phase 5** implements the full module system from §11: file-to-module mapping, `pub`
visibility, import forms, orphan rule enforcement at Gate 3, `.λo` format with five
sections, export-table hashing, and the four-step link procedure.

**Phase 6** implements the stdlib from §16: `Std.Prelude`, `Std.String`, `Std.List`,
`Std.Map`, `Std.Show`, `Std.IO`. Each stdlib module is a normal `.λ` source file
compiled through the same pipeline as user code.

---

## 13. Open Questions

### Resolved (from v2.3)

1. ✅ **Substitution lemma** — Proven by induction on typing derivation
2. ✅ **Preservation** — Verified: well-typed terms preserve type under reduction
3. ✅ **Progress** — Verified: well-typed terms don't get stuck
4. ✅ **Linearity** — Verified: multiplicity rules enforced correctly
5. ✅ **S5′** — Verified: concurrency safety formal properties established

### Resolved (this version, v2.4)

6. ✅ **Module granularity** — One file, one module; `module` keyword reserved for v2.0
7. ✅ **Default visibility** — Private by default; `pub` for explicit export
8. ✅ **Opaque types** — `pub type T` = opaque; `pub type T(..)` = transparent
9. ✅ **Import discipline** — Qualified by default; selective/full unqualified as opt-in
10. ✅ **Cyclic imports** — Static error; module graph must be a DAG
11. ✅ **Orphan rule** — impl legal only in module defining the trait or the type
12. ✅ **Impl visibility** — Impls are implicit on type/trait import; never explicitly imported or re-exported
13. ✅ **`.λo` format** — MessagePack, five sections, export-table content hash
14. ✅ **Incremental recompilation** — Hash-based; type-section stability determines propagation
15. ✅ **Net section format** — Full nets in v1.0; templated nets deferred to v2.0
16. ✅ **Two-phase S5′** — Per-module at Gate 5, global at Link Step L3
17. ✅ **Prelude contents** — Minimal and stable (see §16.1)
18. ✅ **Clone/Drop as load-bearing traits** — Tied to δ/ε agent semantics
19. ✅ **Float/Hash exclusion** — NaN breaks Hash contract; intentional omission
20. ✅ **IO model** — Capability-based; `IO` token at `&` multiplicity
21. ✅ **File linearity** — `File` is linear (`_1`); not closing is a linearity violation

22. ✅ **Grammar split** — §2 divided into concrete surface grammar (§2.A) and theoretical calculus (§2.B)
23. ✅ **Ops as trait methods** — binary/unary ops removed from grammar; defined as `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` trait impls in prelude
24. ✅ **Div/Rem totality** — `div` and `rem` return `Result α DivisionByZero`; no runtime panic
25. ✅ **Boolean logic** — `not`, `and`, `or` are strict prelude functions; short-circuit deferred to v2.0
26. ✅ **Constructor patterns** — deferred to Phase 5a; current parser handles `_` and identifier only
27. ✅ **Naming convention** — lowercase = type variable, uppercase = type constructor; enforced by parser
28. ✅ **Type application syntax** — `type_app ::= type_atom { type_atom }` mirroring term-level juxtaposition
29. ✅ **`&` in type position** — parsed as reference type prefix; distinguished from multiplicity `&` by context
30. ✅ **`impl C for T` syntax** — trait-first ordering; consistent with functional language conventions
31. ✅ **Grammar extensions as Phase 5a** — identified as hard prerequisite; Phases 5 and 6 blocked on it
32. ✅ **Primitive evaluation model** — Options A+C: `Prim(op)` and `PrimVal` agents in the net (A); user code reaches them only through `prim_*` intrinsic wrappers in the prelude (C)
33. ✅ **PrimVal payload** — typed: `PrimVal(Int, 42)` — redundant for correct programs but required for debugging and translator validation
34. ✅ **Prim agent arity** — fixed per operation: 3-port for binary ops, 2-port for unary ops; matches hardware structure, avoids double-firing overhead
35. ✅ **Intrinsics set** — 39 intrinsics total (30 arithmetic + 9 IO); closed and final for v1.0; any `prim_*` name not in the table is `UnknownPrimitive` at Gate 5
36. ✅ **Hybrid evaluator eliminated** — uniform evaluator loop handles `Prim ⋈ PrimVal` rules identically to `λ ⋈ @`; no special arithmetic evaluation mode
37. ✅ **IO is a monad** — `IO a` describes an effectful computation; no capability threading in user code
38. ✅ **IO location** — `IO` type + `Monad IO` instance + all operations in `Std.IO`; orphan rule satisfied
39. ✅ **Monad trait hierarchy** — full `Functor → Applicative → Monad` in prelude; `Option` and `Result` get monad instances for free
40. ✅ **IO sequencing** — internal `IO_token` linear agent threads through `PrimIO` rules; sequencing enforced by data dependency in the net, not by a scheduler
41. ✅ **`do`-notation** — deferred to v1.1; v1.0 uses explicit `bind` and `then`
42. ✅ **File linearity** — `File` is linear; `close` is the only consumer; forgetting to close is a compile-time `LinearityViolation`
43. ✅ **`prim_io_*` wire-up** — 9 IO intrinsics defined in §16.3.2 with full table; translator threads `IO_token` wire through each `prim_io_*` call; `bind` in `Monad IO` desugars to sequential token threading

### Open (deferred to v2.0)
- **FFI** — Interfacing with native code
- **Dynamic module loading** — Plugin system
- **Versioning** — Disambiguating two versions of the same module
- **Templated nets** — Named-port `.λo` format for zero-duplication linking
- **Mutable maps / arrays** — Requires allocator design and `&`-mutation semantics
- **Short-circuit `and`/`or`** — Requires lazy semantics not available in interaction nets
- **Error message quality** — Detail level for type errors
- **Debug format** — DWARF-compatible debug info

### Open (deferred to v1.1)
- **`do`-notation** — Syntactic sugar for nested `bind`/`then` chains

---

## 14. Trade-offs

| Decision | Rationale |
|----------|-----------|
| No garbage collector | ε-agents provide automatic memory management |
| No runtime type checks | Type system guarantees safety at compile time |
| Conservative S5′ | May over-constrain some safe parallel programs (§7.5) |
| Borrow mode cannot be duplicated | Prevents aliasing of mutable references |
| ω implies immutability | Shared references cannot be mutated (prevents data races) |
| Private by default | Prevents accidental coherence violations across module boundaries |
| Module graph must be DAG | Makes global coherence and S5′ decidable without whole-program analysis |
| Full nets in .λo (v1.0) | Simpler linker at the cost of potential duplication; templated nets in v2.0 |
| Capability-based IO | Fits the existing `&` mode; avoids introducing monads as a language concept |
| File is linear | Forgetting to close a file is a compile-time error, not a resource leak |
| Float does not implement Hash | NaN breaks the Hash/Eq contract; exclusion is safer than a broken impl |
| Minimal prelude | Breaking prelude changes are breaking language changes; start small |
| Show in stdlib, not prelude | Show returns String; String is in stdlib; prelude cannot depend on stdlib |
| Ops are trait methods, not syntax | Keeps the grammar minimal; ops are functions with types and multiplicities |
| Div returns Result | Division by zero is a total function; no runtime panics from the type system |
| and/or are strict | Short-circuit requires lazy semantics not available in interaction nets (v1.0) |
| Prim agents in the net (Option A+C) | Uniform evaluator loop; no hybrid evaluation mode; net is the sole semantic model |
| PrimVal carries type tag | Redundant for correct programs but enables debugging and translator validation |
| Fixed arity for Prim agents | Efficient: binary ops fire in one rule; matches hardware; avoids double-firing overhead |
| 30 arithmetic + 9 IO intrinsics, closed set | Prevents user code from forging primitives; arithmetic and IO clearly separated by agent class |
| IO intrinsics use IO_token port, arithmetic do not | Arithmetic is pure and parallelisable; IO is sequenced by data dependency — same evaluator loop, different port structure |
| IO is a monad, not a capability | Cleaner composition; no explicit IO threading in user code; established functional language model |
| Functor/Applicative/Monad in prelude | Option and Result become monads for free; IO monad instance in Std.IO satisfies orphan rule |
| IO_token internal, not surface | User sees clean `IO a` type; sequencing enforced by net data dependency without exposing the token |
| do-notation deferred to v1.1 | Grammar extension not required for correctness; explicit bind is sufficient for v1.0 |
| File has independent IO_token | Multiple files can be operated on concurrently in the net; no cross-file sequencing constraints |

---

## 15. Phase 0: Lean4 Formalization

> **This section documents the completed Phase 0 formal verification in Lean4.**

### 15.1 Overview

Phase 0 establishes the mathematical foundation of λ◦₀ (the core calculus without
traits, native types, or match/view) through verified proofs in Lean4. This ensures the
type system is sound before any implementation work begins.

### 15.2 Module Structure

```
lean4/
├── lakefile.lean
├── LambdaCalculus.lean                    # Root import
└── LambdaCalculus/
    ├── Core/
    │   ├── Semiring.lean                   # Quantity {0,1,ω}, Mode {&}
    │   ├── Context.lean                    # Context operations
    │   └── Terms.lean                      # Type, Term, Value, Substitution
    ├── TypeSystem/
    │   └── TypingRules.lean                # HasType relation, substitution lemma
    ├── Translation/
    │   └── NetTranslation.lean             # Term → Net translation
    └── Metatheory/
        ├── Preservation.lean               # Type preservation under reduction
        ├── Progress.lean                   # Well-typed terms don't get stuck
        ├── Linearity.lean                  # Multiplicity enforcement
        └── S5Prime.lean                    # Concurrency safety formalization
```

### 15.3 Key Formalizations

#### 15.3.1 Quantity Semiring (Semiring.lean)

The quantity axis is formalized as a commutative semiring `(Q, +, ·, 0, 1)`:

```lean
inductive Quantity
  | zero : Quantity  -- 0: erased
  | one  : Quantity  -- 1: linear
  | omega : Quantity -- ω: shared

def Quantity.add : Quantity → Quantity → Quantity
def Quantity.mul : Quantity → Quantity → Quantity
```

**Proved properties:**
- `zero_add`: `0 + q = q`
- `one_mul`: `1 · q = q`
- `add_comm`: `q₁ + q₂ = q₂ + q₁`
- `add_assoc`: `(q₁ + q₂) + q₃ = q₁ + (q₂ + q₃)`
- `mul_assoc`: `(q₁ · q₂) · q₃ = q₁ · (q₂ · q₃)`
- `left_distrib`: `q₁ · (q₂ + q₃) = (q₁ · q₂) + (q₁ · q₃)`
- `right_distrib`: `(q₁ + q₂) · q₃ = (q₁ · q₃) + (q₂ · q₃)`
- `omega_absorb`: `ω + q = ω`
- `zero_mul`: `0 · q = 0`

#### 15.3.2 Mode (Borrow)

```lean
inductive Mode
  | borrow : Mode  -- &: borrowed reference (observer)

inductive Multiplicity
  | qty (q : Quantity) : Multiplicity
  | mode (m : Mode) : Multiplicity
```

#### 15.3.3 Context Operations (Context.lean)

Critical formalization of context operations:

```lean
structure Binding where
  name : String
  mult : Quantity
  type : Type

abbrev Context := List Binding
```

**Key operations:**

```lean
def Context.addCtx (Γ₁ Γ₂ : Context) : Context
-- Pointwise addition of contexts (context splitting)

def Context.scale (q : Quantity) (Γ : Context) : Context
-- Scalar multiplication: scales all entries
-- CRITICAL: 0 · Γ produces _0-annotated context, NOT empty

def Context.weaken (Γ : Context) (x : String) (τ : Type) : Context
-- Add a _0 binding to the context
```

**Proved property:**
- `scale_distributes`: `q · (Γ₁ + Γ₂) = (q · Γ₁) + (q · Γ₂)`

#### 15.3.4 Term Syntax (Terms.lean)

```lean
inductive Type
  | native : NativeKind → Type
  | arrow : Quantity → Type → Type → Type
  | forallE : String → Type → Type
  | constraint : TraitName → Type → Type
  | inductive : String → List Type → Type
  | borrow : Type → Type
  | product : Type → Type → Type
  | sum : Type → Type → Type

inductive Term
  | var : String → Term
  | abs : String → Quantity → Type → Term → Term
  | app : Term → Term → Term
  | letTerm : String → Quantity → Type → Term → Term → Term
```

#### 15.3.5 Typing Rules (TypingRules.lean)

```lean
inductive HasType : Context → Term → Type → Prop
  | var {Γ : Context} {x : String} {q : Quantity} {τ : Type} :
      HasType (Context.add Γ {name := x, mult := q, type := τ}) (Term.var x) τ
  | varOmega {Γ : Context} {x : String} {τ : Type} :
      HasType Γ (Term.var x) τ
  | varBorrow {Γ : Context} {x : String} {τ : Type} :
      HasType Γ (Term.var x) (Type.borrow τ)
  | abs {Γ : Context} {x : String} {q : Quantity} {τ₁ τ₂ : Type} {e : Term} :
      HasType (Context.add Γ {name := x, mult := q, type := τ₁}) e τ₂ →
      HasType Γ (Term.abs x q τ₁ e) (Type.arrow q τ₁ τ₂)
  | app {Γ₁ Γ₂ : Context} {e₁ e₂ : Term} {q : Quantity} {τ₂ τ : Type} :
      HasType Γ₁ e₁ (Type.arrow q τ₂ τ) →
      HasType Γ₂ e₂ τ₂ →
      HasType (Context.addCtx Γ₁ (Context.scale q Γ₂)) (Term.app e₁ e₂) τ
  | letTerm {Γ₁ Γ₂ : Context} {x : String} {q : Quantity} {τ₁ τ₂ : Type} {e₁ e₂ : Term} :
      HasType Γ₁ e₁ τ₁ →
      HasType (Context.add Γ₂ {name := x, mult := q, type := τ₁}) e₂ τ₂ →
      HasType (Context.addCtx Γ₁ (Context.scale q Γ₂)) (Term.letTerm x q τ₁ e₁ e₂) τ₂
  | weaken {Γ : Context} {x : String} {e : Term} {τ τ' : Type} :
      HasType Γ e τ →
      HasType (Context.weaken Γ x τ') e τ
```

#### 15.3.6 Substitution Lemma (TypingRules.lean)

**Lemma**: If `Γ₁, x:q,τ₁ ⊢ e : τ₂` and `Γ₂ ⊢ v : τ₁`, then
`Γ₁ + q·Γ₂ ⊢ e[v/x] : τ₂`.

**Proof**: By induction on the typing derivation. Cases:
- **Var**: Trivial when `x ≠ y`; when `x = y`, follows from `Γ₂ ⊢ v : τ₁`
- **Var-Ω, Var-Borrow**: Similar to Var
- **Abs**: Inductive hypothesis on body; rebuild abstraction
- **App**: Critical — split contexts correctly using `addCtx` and `scale`
- **Let**: Critical — scale the outer context by `q` correctly
- **Weaken**: Variable being substituted is different from weakening variable

#### 15.3.7 Preservation Theorem (Preservation.lean)

**Theorem**: If `Γ ⊢ e : τ` and `e ⟶ e'`, then `Γ ⊢ e' : τ`.

**Proof**: By cases on the reduction rule. The β-reduction case uses the substitution
lemma.

#### 15.3.8 Progress Theorem (Progress.lean)

**Theorem**: If `∅ ⊢ e : τ`, then either `e` is a value or there exists `e'` such
that `e ⟶ e'`.

**Proof**: By induction on the typing derivation. Cases:
- **Var**: Impossible — empty context has no bindings
- **Abs**: Value (function)
- **App**: Either `e₁` steps, or `e₂` steps, or β-reduce if both values
- **Let**: Similar to App

#### 15.3.9 Linearity (Linearity.lean)

Theorems about correct multiplicity usage:
- Linear variables (`_1`) are used exactly once
- Erased variables (`_0`) are never used
- Shared variables (`_ω`) may be used multiple times

#### 15.3.10 Net Translation (NetTranslation.lean)

```lean
inductive NetAgent
  | lambda : NetAgent      -- λ
  | app : NetAgent         -- @
  | delta : NetAgent       -- δ (duplication)
  | epsilon : NetAgent     -- ε (erasure)
  | constructor : String → NetAgent

inductive Net
  | node : NetAgent → List Port → Net
  | wire : Port → Port → Net
  | par : Net → Net → Net   -- parallel composition
```

**Translation policy:**
- `_1` bindings → direct wire connection
- `_ω` bindings → insert δ-agent (duplication)
- `_0` bindings → insert ε-agent (erasure)
- `_&` bindings → observe-only connection

#### 15.3.11 S5' Concurrency Safety (S5Prime.lean)

```lean
inductive QuantModality
  | Zero : QuantModality    -- □ (necessary)
  | One : QuantModality     -- (linear)
  | Omega : QuantModality   -- ◇ (possible)

-- S5' theorem: If a net is S5'-safe, parallel evaluation is race-free
```

### 15.4 Build Verification

```bash
cd lean4 && LAKE_BUILD_CACHE=false lake build
```

**Status**: ✅ All proofs passing

---

## 16. Standard Library & Prelude

> **This section is new in v2.4.**

### 16.1 Design Principles

Every stdlib module is a normal `.λ` source file compiled through the same six-gate
pipeline as user code. There is no compiler magic beyond the prelude auto-import. All
stdlib functions are **honest about multiplicity**: a function that consumes a value
takes it at `_1`; a function that observes takes it at `_&`; a function that needs
repeated use requires `_ω` or `_&`.

**The prelude is a language contract.** Removing or changing a prelude export is a
breaking change to the language, not to a library. The prelude must therefore be minimal
and stable.

**The five-layer hierarchy:**

```
Layer 0 — Std.Prelude   (auto-imported; no deps)
           └── prim_*   (compiler intrinsics; not importable)
Layer 1 — Std.String    (close to native types)
           Std.Show     (depends on Std.String)
Layer 2 — Std.List      (built on Prelude)
           Std.Map      (built on Std.List, Ord)
           Std.Set      (thin wrapper on Std.Map)
Layer 3 — Std.IO        (depends on all of the above)
```

A module in Layer N may not import from Layer N+1 or higher. This ordering is enforced
by the DAG check at Gate 2 and is also the correct compilation order.

### 16.2 `Std.Prelude` — Auto-imported

The prelude is implicitly available in every module as `use Std.Prelude (..)`. It
exports exactly the following.

#### Types and Constructors

```
pub type Bool (..)          -- True | False
pub type Unit (..)          -- {} (zero-width)
pub type Option α (..)      -- None | Some α
pub type Result α ε (..)    -- Ok α | Err ε
```

`Int`, `Float`, and `Char` are compiler-defined native types available everywhere
without import. They are not defined in any `.λ` source file.

#### Core Traits

**Equality and ordering:**

```
pub trait Eq α where
  val eq  : &α →¹ &α →¹ Bool
  val neq : &α →¹ &α →¹ Bool     -- default: not (eq x y)

pub trait Ord α where Eq α
  val compare : &α →¹ &α →¹ Ordering
  val lt  : &α →¹ &α →¹ Bool
  val gt  : &α →¹ &α →¹ Bool
  val lte : &α →¹ &α →¹ Bool
  val gte : &α →¹ &α →¹ Bool

pub type Ordering (..) = LT | EQ | GT

pub trait Hash α where Eq α
  val hash : &α →¹ Int
```

**Memory and lifetime traits (load-bearing — tied to δ/ε agent semantics):**

```
pub trait Clone α where
  val clone : &α →¹ α             -- δ-agent calls this for ω bindings

pub trait Copy α where Clone α    -- marker: clone is zero-cost (natives only)

pub trait Drop α where
  val drop : α →¹ Unit            -- ε-agent calls this before erasure

pub trait Sized α                  -- marker: compile-time known size
```

**`Clone` and `Drop` are load-bearing.** A type at multiplicity `ω` must implement
`Clone` — the δ-agent needs to know how to duplicate it. A linear type (`_1`) leaving
scope without being consumed triggers `Drop`. These traits cannot be omitted from the
prelude.

**Arithmetic traits:**

Binary and unary operations on native numeric types are provided as prelude trait
implementations. They are ordinary functions applied via juxtaposition (see §2.A.1).

```
pub trait Add α where
  val add : α →¹ α →¹ α

pub trait Sub α where
  val sub : α →¹ α →¹ α

pub trait Mul α where
  val mul : α →¹ α →¹ α

pub trait Div α where
  val div : α →¹ α →¹ Result α DivisionByZero   -- total: no runtime panic

pub trait Rem α where
  val rem : α →¹ α →¹ Result α DivisionByZero

pub trait Neg α where              -- unary negation
  val neg : α →¹ α

pub type DivisionByZero (..) = DivisionByZero   -- unit-like error type
```

**`Div` and `Rem` return `Result`.** Integer division by zero is not a runtime panic —
it is a total function returning `Err DivisionByZero`. This is consistent with the
constraint that no runtime panics originate from the type system (§1). The caller must
handle the error via `match`.

**Boolean logic** is provided as standalone prelude functions (not a trait, since `Bool`
is the only meaningful instance):

```
pub val not : Bool →¹ Bool
pub val and : Bool →¹ Bool →¹ Bool    -- strict (not short-circuit)
pub val or  : Bool →¹ Bool →¹ Bool    -- strict (not short-circuit)
```

Short-circuit evaluation requires lazy semantics not available in the current
interaction net model. `and` and `or` are strict; lazy variants are deferred to v2.0.

**Effect abstraction traits (Functor → Applicative → Monad hierarchy):**

These traits abstract over type constructors `f : Type -> Type` that represent
computational contexts. They are defined in the prelude so that `Option` and `Result`
instances are available everywhere, and so that `Std.IO` can provide a `Monad IO`
instance by the orphan rule (IO is defined in `Std.IO`).

```
pub trait Functor f where
  val fmap : (a -> b) -> f a -> f b
  -- Laws: fmap id = id
  --       fmap (g . h) = fmap g . fmap h

pub trait Applicative f where Functor f
  val pure  : a -> f a
  val apply : f (a -> b) -> f a -> f b
  -- Laws: pure id <*> v = v
  --       pure f <*> pure x = pure (f x)

pub trait Monad f where Applicative f
  val bind : f a -> (a -> f b) -> f b
  val then : f a -> f b -> f b     -- default: bind x (λ_:0:a. y)
  -- Laws: bind (pure x) f = f x
  --       bind m pure = m
  --       bind (bind m f) g = bind m (λx. bind (f x) g)
```

**Instances for prelude types:**

```
-- Option: failure-propagating monad (flatMap semantics)
impl Functor Option
impl Applicative Option
impl Monad Option
-- bind (Some x) f = f x
-- bind None     _ = None

-- Result: error-propagating monad
impl Functor (Result ε)
impl Applicative (Result ε)
impl Monad (Result ε)
-- bind (Ok x)  f = f x
-- bind (Err e) _ = Err e
```

**`Monad IO` is not in the prelude.** `IO` is defined in `Std.IO` (Layer 3). By the
orphan rule (§8.4), `impl Monad IO` must live in `Std.IO`. The prelude defines the
trait; `Std.IO` provides the instance.

#### Native Type Impl Matrix

| Type | Eq | Ord | Hash | Clone | Copy | Drop | Add | Sub | Mul | Div | Rem | Neg |
|------|----|-----|------|-------|------|------|-----|-----|-----|-----|-----|-----|
| `Int` | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `Float` | ✅ | ✅ | — | ✅ | ✅ | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `Bool` | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | — | — | — | — | — |
| `Char` | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | — | — | — | — | — |
| `Unit` | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | — | — | — | — | — |

`Float` does not implement `Hash` — NaN breaks the `a == b → hash(a) == hash(b)`
contract (§9.3). `Drop` is omitted for all native scalars — they have no heap
allocation; the ε-agent erases them directly. `Bool`, `Char`, and `Unit` do not
implement arithmetic traits — no meaningful numeric semantics apply.

#### Conditional Impls for Prelude Types

```
impl Eq (Option α) where Eq α
impl Ord (Option α) where Ord α
impl Clone (Option α) where Clone α
impl Functor Option
impl Applicative Option
impl Monad Option

impl Eq (Result α ε) where Eq α, Eq ε
impl Clone (Result α ε) where Clone α, Clone ε
impl Functor (Result ε)
impl Applicative (Result ε)
impl Monad (Result ε)

impl Eq Ordering
impl Ord Ordering
impl Clone Ordering
impl Copy Ordering
```

**Why is `Show` absent from the prelude?** `Show` returns `String`. `String` is defined
in `Std.String` (Layer 1). A prelude trait cannot depend on a stdlib type. `Show` lives
in `Std.Show` (see §16.6).

### 16.3 Compiler Intrinsics (`prim_*`)

> **This subsection is implementation documentation, not user-facing language.**
> No λ◦ source program may reference `prim_*` names directly. Doing so produces
> an `UnknownPrimitive` error at Gate 5 unless the name is in the table below.

The closed intrinsics set is the bridge between the trait method surface (§16.2) and
the `Prim(op)` / `PrimVal` agents and the `PrimIO(op)` / `IO_token` agents in the
interaction net (§6.1). It is divided into two groups:

- **§16.3.1 — Arithmetic intrinsics** (30): pure, order-independent, emit `Prim` agents
- **§16.3.2 — IO intrinsics** (9): effectful, order-enforced by `IO_token`, emit `PrimIO` agents

**Total: 39 intrinsics.** This set is closed and final for v1.0.

### 16.3.1 Arithmetic Intrinsics

**Pure — order-independent — emit `Prim` agents.** These intrinsics can fire in any
order, including in parallel. The result is always deterministic.

#### Integer

| Intrinsic | Arity | Agent | Prelude wrapper | Result type |
|-----------|-------|-------|-----------------|-------------|
| `prim_iadd` | binary | `Prim(IAdd)` | `Add.add : Int -> Int -> Int` | `Int` |
| `prim_isub` | binary | `Prim(ISub)` | `Sub.sub : Int -> Int -> Int` | `Int` |
| `prim_imul` | binary | `Prim(IMul)` | `Mul.mul : Int -> Int -> Int` | `Int` |
| `prim_idiv` | binary | `Prim(IDiv)` | `Div.div : Int -> Int -> Result Int DivisionByZero` | `Result Int DivisionByZero` |
| `prim_irem` | binary | `Prim(IRem)` | `Rem.rem : Int -> Int -> Result Int DivisionByZero` | `Result Int DivisionByZero` |
| `prim_ineg` | unary | `Prim(INeg)` | `Neg.neg : Int -> Int` | `Int` |
| `prim_ieq` | binary | `Prim(IEq)` | `Eq.eq : &Int -> &Int -> Bool` | `Bool` |
| `prim_ilt` | binary | `Prim(ILt)` | `Ord.lt : &Int -> &Int -> Bool` | `Bool` |
| `prim_igt` | binary | `Prim(IGt)` | `Ord.gt : &Int -> &Int -> Bool` | `Bool` |
| `prim_ile` | binary | `Prim(ILe)` | `Ord.lte : &Int -> &Int -> Bool` | `Bool` |
| `prim_ige` | binary | `Prim(IGe)` | `Ord.gte : &Int -> &Int -> Bool` | `Bool` |
| `prim_ihash` | unary | `Prim(IHash)` | `Hash.hash : &Int -> Int` | `Int` |

#### Float

| Intrinsic | Arity | Agent | Prelude wrapper | Result type |
|-----------|-------|-------|-----------------|-------------|
| `prim_fadd` | binary | `Prim(FAdd)` | `Add.add : Float -> Float -> Float` | `Float` |
| `prim_fsub` | binary | `Prim(FSub)` | `Sub.sub : Float -> Float -> Float` | `Float` |
| `prim_fmul` | binary | `Prim(FMul)` | `Mul.mul : Float -> Float -> Float` | `Float` |
| `prim_fdiv` | binary | `Prim(FDiv)` | `Div.div : Float -> Float -> Result Float DivisionByZero` | `Result Float DivisionByZero` |
| `prim_frem` | binary | `Prim(FRem)` | `Rem.rem : Float -> Float -> Result Float DivisionByZero` | `Result Float DivisionByZero` |
| `prim_fneg` | unary | `Prim(FNeg)` | `Neg.neg : Float -> Float` | `Float` |
| `prim_feq` | binary | `Prim(FEq)` | `Eq.eq : &Float -> &Float -> Bool` | `Bool` |
| `prim_flt` | binary | `Prim(FLt)` | `Ord.lt : &Float -> &Float -> Bool` | `Bool` |
| `prim_fgt` | binary | `Prim(FGt)` | `Ord.gt : &Float -> &Float -> Bool` | `Bool` |
| `prim_fle` | binary | `Prim(FLe)` | `Ord.lte : &Float -> &Float -> Bool` | `Bool` |
| `prim_fge` | binary | `Prim(FGe)` | `Ord.gte : &Float -> &Float -> Bool` | `Bool` |

#### Bool

| Intrinsic | Arity | Agent | Prelude wrapper | Result type |
|-----------|-------|-------|-----------------|-------------|
| `prim_bnot` | unary | `Prim(BNot)` | `not : Bool -> Bool` | `Bool` |
| `prim_band` | binary | `Prim(BAnd)` | `and : Bool -> Bool -> Bool` | `Bool` |
| `prim_bor` | binary | `Prim(BOr)` | `or : Bool -> Bool -> Bool` | `Bool` |
| `prim_beq` | binary | `Prim(BEq)` | `Eq.eq : &Bool -> &Bool -> Bool` | `Bool` |
| `prim_bhash` | unary | `Prim(BHash)` | `Hash.hash : &Bool -> Int` | `Int` |

#### Char

| Intrinsic | Arity | Agent | Prelude wrapper | Result type |
|-----------|-------|-------|-----------------|-------------|
| `prim_ceq` | binary | `Prim(CEq)` | `Eq.eq : &Char -> &Char -> Bool` | `Bool` |
| `prim_cord` | unary | `Prim(COrd)` | `Ord.compare : &Char -> &Char -> Ordering` | `Ordering` |
| `prim_chash` | unary | `Prim(CHash)` | `Hash.hash : &Char -> Int` | `Int` |

**Subtotal: 30 arithmetic intrinsics.**

**Example — how `add 3 5` evaluates end-to-end:**

```
-- 1. User writes:
add 3 5

-- 2. Name resolution: `add` resolves to Std.Prelude.Add.add for Int

-- 3. Prelude definition:
val add : Int -> Int -> Int = λx:1:Int. λy:1:Int. prim_iadd x y

-- 4. Net translation emits:
@( @( λx. λy. Prim(IAdd)[aux1=x, aux2=y],  PrimVal(Int,3) ),  PrimVal(Int,5) )

-- 5. Evaluator fires β-reductions (two @ ⋈ λ rules):
Prim(IAdd)[aux1=PrimVal(Int,3), aux2=PrimVal(Int,5)]

-- 6. Evaluator fires primitive rule (Prim(IAdd) ⋈ PrimVal ⋈ PrimVal):
PrimVal(Int, 8)

-- 7. Result: the value 8
```

The evaluator loop is uniform throughout — steps 5 and 6 use the same "find active
pair, fire rule" mechanism. No special cases.

### 16.3.2 IO Intrinsics

**Effectful — order-enforced by `IO_token` — emit `PrimIO` agents.** Unlike arithmetic
intrinsics, IO intrinsics are not pure. Their firing order is determined by the
`IO_token` linear chain: each `PrimIO` agent consumes the incoming token and produces a
fresh one, forcing sequential execution through data dependency. The parallel evaluator
cannot reorder IO actions because the next action's token input depends on the previous
action's token output.

These intrinsics are the bridge between `Std.IO` wrapper functions (§16.8) and the
`PrimIO(op)` agents in the net (§6.1). They are emitted by the net translator when it
encounters a `prim_io_*` call inside a `Std.IO` definition.

| Intrinsic | Arity | Agent | `Std.IO` wrapper | Result type |
|-----------|-------|-------|-----------------|-------------|
| `prim_io_print` | unary | `PrimIO(Print)` | `print : &String -> IO Unit` | `IO Unit` |
| `prim_io_println` | unary | `PrimIO(Println)` | `println : &String -> IO Unit` | `IO Unit` |
| `prim_io_eprint` | unary | `PrimIO(EPrint)` | `eprint : &String -> IO Unit` | `IO Unit` |
| `prim_io_eprintln` | unary | `PrimIO(EPrintln)` | `eprintln : &String -> IO Unit` | `IO Unit` |
| `prim_io_read_line` | nullary | `PrimIO(ReadLine)` | `read_line : IO (Result String IOError)` | `IO (Result String IOError)` |
| `prim_io_open` | unary | `PrimIO(Open)` | `open : &String -> IO (Result File IOError)` | `IO (Result File IOError)` |
| `prim_io_close` | unary | `PrimIO(Close)` | `close : File ->¹ IO Unit` | `IO Unit` |
| `prim_io_read` | unary | `PrimIO(Read)` | `read : &File -> IO (Result String IOError)` | `IO (Result String IOError)` |
| `prim_io_write` | binary | `PrimIO(Write)` | `write : &File -> &String -> IO (Result Unit IOError)` | `IO (Result Unit IOError)` |

**Subtotal: 9 IO intrinsics. Grand total: 39 intrinsics.**

**Arity note for `prim_io_read_line`**: nullary means it takes no value argument —
only the `IO_token` port. The `PrimIO(ReadLine)` agent is 2-port: principal +
`IO_token`.

**`prim_io_close` consumes its `File` argument at multiplicity `_1`.** The translator
verifies the `File` value is not used after `close` is called. This is enforced by the
borrow checker at Gate 4 before the net is ever constructed.

**Example — how `println "hello"` evaluates end-to-end:**

```
-- 1. User writes (inside an IO monad context):
println "hello"

-- 2. Name resolution: `println` resolves to Std.IO.println

-- 3. Std.IO definition:
val println : &String -> IO Unit =
  λs:&:String. prim_io_println s

-- 4. IO monad lowers IO Unit to: IO_token ->¹ (IO_token, Unit)
--    Net translation emits:
@( λs. PrimIO(Println)[token=IO_token_in, aux=s],  PrimVal(String,"hello") )

-- 5. Evaluator fires β-reduction (@ ⋈ λ):
PrimIO(Println)[token=IO_token_in, aux=PrimVal(String,"hello")]

-- 6. Evaluator fires IO rule (PrimIO(Println) ⋈ IO_token ⋈ PrimVal(String,...)):
--    Side effect: writes "hello\n" to stdout
--    Produces: (IO_token_new, PrimVal(Unit, {}))

-- 7. IO_token_new flows to the next IO action in the bind chain
--    PrimVal(Unit, {}) flows to the result of this action
```

The key difference from arithmetic: step 6 executes a real side effect. The token
`IO_token_new` produced in step 6 is what gates the next IO action — it cannot fire
until this one completes.

### 16.4 `Std.String`

```
use Std.String
```

`String` is a **linear type** (`_1`) by default — owned and consumed on use. Shared
strings use `Clone`. A `&String` is a borrowed view into an existing string with no
allocation.

```
pub type String                   -- opaque (constructors not exported)

-- Construction
pub val empty      : String
pub val from_chars : List Char →¹ String

-- Observation (borrow string, return owned values)
pub val length     : &String →¹ Int
pub val is_empty   : &String →¹ Bool
pub val chars      : &String →¹ List Char      -- allocates new list

-- Transformation (consume and produce)
pub val append     : String →¹ String →¹ String
pub val slice      : &String →¹ Int →¹ Int →¹ &String   -- zero-copy view
pub val to_upper   : String →¹ String
pub val to_lower   : String →¹ String

-- Search (observe only)
pub val contains    : &String →¹ &String →¹ Bool
pub val starts_with : &String →¹ &String →¹ Bool
pub val ends_with   : &String →¹ &String →¹ Bool
pub val split       : &String →¹ Char →¹ List &String    -- views into original

-- Trait impls
impl Eq String
impl Ord String
impl Hash String
impl Clone String
```

### 16.5 `Std.List`

```
use Std.List
```

`List` is a singly-linked inductive type. The multiplicity of the element type `α`
propagates through the list.

```
pub type List α (..) = Nil | Cons α (List α)

-- Construction
pub val nil       : List α
pub val cons      : α →¹ List α →¹ List α
pub val singleton : α →¹ List α

-- Deconstruction (ownership transfer — consumes spine)
pub val head   : List α →¹ Option α
pub val tail   : List α →¹ Option (List α)
pub val uncons : List α →¹ Option (α, List α)

-- Observation (borrow — no consumption)
pub val length   : &(List α) →¹ Int
pub val is_empty : &(List α) →¹ Bool
pub val head_ref : &(List α) →¹ Option &α

-- Transformation (consume and produce)
pub val map        : (α →¹ β) →¹ List α →¹ List β
pub val filter     : (&α →¹ Bool) →¹ List α →¹ List α
pub val fold_left  : (β →¹ α →¹ β) →¹ β →¹ List α →¹ β
pub val fold_right : (α →¹ β →¹ β) →¹ List α →¹ β →¹ β
pub val append     : List α →¹ List α →¹ List α
pub val reverse    : List α →¹ List α
pub val zip        : List α →¹ List β →¹ List (α, β)
pub val take       : Int →¹ List α →¹ List α
pub val drop       : Int →¹ List α →¹ List α

-- Trait impls
impl Eq (List α) where Eq α
impl Ord (List α) where Ord α
impl Clone (List α) where Clone α
```

**Note on `map`**: the function argument is `α →¹ β`, consuming each element exactly
once. This is safe for lists of linear values. Since `ω` is a supertype of `1` under
the semiring order, passing an `ω`-function is always valid.

### 16.6 `Std.Show`

`Show` cannot be in the prelude (§16.2). It lives here as a minimal module with no
dependencies beyond `Std.String`.

```
use Std.Show
```

```
pub trait Show α where
  pub val show : &α →¹ String

-- Native type instances
impl Show Int
impl Show Float
impl Show Bool
impl Show Char
impl Show Unit

-- Conditional instances
impl Show Ordering
impl Show (Option α) where Show α
impl Show (Result α ε) where Show α, Show ε
impl Show (List α) where Show α
impl Show String                              -- wraps in double quotes
```

### 16.7 `Std.Map`

```
use Std.Map
```

`Map` is a **persistent** ordered map. `insert` and `delete` return new maps rather than
mutating in place. This is the only design consistent with the `ω`-implies-immutability
invariant (§14) — a shared map cannot be mutated without a data race. Keys must satisfy
`Ord`.

```
pub type Map κ ν               -- opaque (constructors not exported)

-- Construction
pub val empty     : Map κ ν
pub val singleton : κ →¹ ν →¹ Map κ ν

-- Insertion / deletion (persistent: returns new map)
pub val insert : κ →¹ ν →¹ Map κ ν →¹ Map κ ν    -- replaces existing key
pub val delete : &κ →¹ Map κ ν →¹ Map κ ν         -- borrows key

-- Lookup
pub val get    : &κ →¹ &(Map κ ν) →¹ Option &ν         -- borrow lookup
pub val remove : &κ →¹ Map κ ν →¹ (Option ν, Map κ ν)  -- ownership extraction

-- Observation
pub val size     : &(Map κ ν) →¹ Int
pub val contains : &κ →¹ &(Map κ ν) →¹ Bool

-- Transformation
pub val map_vals : (ν →¹ μ) →¹ Map κ ν →¹ Map κ μ
pub val filter   : (&κ →¹ &ν →¹ Bool) →¹ Map κ ν →¹ Map κ ν
pub val fold     : (β →¹ κ →¹ ν →¹ β) →¹ β →¹ Map κ ν →¹ β
pub val keys     : &(Map κ ν) →¹ List &κ
pub val values   : Map κ ν →¹ List ν               -- consumes map

-- Trait impls
impl Eq (Map κ ν) where Eq κ, Eq ν
impl Clone (Map κ ν) where Clone κ, Clone ν
impl Show (Map κ ν) where Show κ, Show ν
```

### 16.8 `Std.IO`

```
use Std.IO
```

#### Design

`IO` is a **monad**. An `IO a` value is a *description* of an effectful computation
that, when executed by the runtime, performs IO and produces a value of type `a`. No
effect occurs until the runtime executes the description. This is the established
functional language model (Haskell, PureScript).

**Purity by type.** A function whose return type does not mention `IO` is guaranteed
pure. The compiler need not inspect the body — the type is the proof.

**Sequencing** is done with `bind` and `then` from the `Monad` trait (§16.2):

```
-- Print two lines in sequence
then (println "hello") (println "world")

-- Read a line and print it back
bind read_line (λline:1:Result String IOError.
  match line with {
    Ok s  => println s
  | Err _ => println "error"
  })
```

**`do`-notation** (syntactic sugar for nested `bind`/`then`) is deferred to v1.1 and
documented in §13 as an open question.

#### Internal Implementation

Internally the compiler lowers `IO a` to a function over a linear `IO_token`:

```
-- Surface type seen by user:
IO a  ≡  (description of a computation producing a)

-- Internal net representation:
IO a  ≡  IO_token ->¹ (IO_token, a)
```

`bind` threads the token from one action to the next, enforcing sequencing by data
dependency in the net. The parallel evaluator cannot reorder IO actions because each
action's output token is the next action's input token. This is the same mechanism as
the `RealWorld` token in GHC's IO implementation, but made explicit in the interaction
net model via the `IO_token` agent (§6.1).

The `IO_token` is created by the runtime at program entry and passed to `main`. It is
discarded when `main` returns. The token cannot be duplicated or erased — it does not
implement `Clone` or `Drop` — so the runtime is the sole creator and final consumer.

#### Type Definition and Monad Instance

```
pub type IO a               -- abstract; internal rep is IO_token ->¹ (IO_token, a)

impl Functor IO
impl Applicative IO
impl Monad IO
-- bind m f = λtoken. let (token', a) = m token in f a token'
-- pure x   = λtoken. (token, x)
```

#### Standard Streams

```
pub val print     : &String -> IO Unit
pub val println   : &String -> IO Unit
pub val eprint    : &String -> IO Unit           -- stderr
pub val eprintln  : &String -> IO Unit           -- stderr with newline
pub val read_line : IO (Result String IOError)   -- stdin; Result handles EOF
```

#### File Operations

`File` is **linear** (`_1`). The only way to eliminate a `File` value is to call
`close`. Forgetting to close is a `LinearityViolation` at Gate 3 — a compile-time
error, not a resource leak. `File` does not implement `Drop`.

Internally, each `File` has its own `IO_token` derived from the main token at `open`
time. File operations thread this per-file token, allowing multiple files to be
operated on concurrently in the net without sequencing constraints between them.

```
pub type File               -- abstract; linear (_1)

pub val open  : &String -> IO (Result File IOError)
pub val close : File ->¹ IO Unit                  -- consumes file (linear)
pub val read  : &File -> IO (Result String IOError)
pub val write : &File -> &String -> IO (Result Unit IOError)
```

#### IO Error Type

```
pub type IOError (..)
  = NotFound
  | PermissionDenied
  | UnexpectedEOF
  | Other String

impl Show IOError
impl Eq IOError
```

#### Complete Usage Example

```
-- Read a file and print its contents, handling errors
let program : IO Unit =
  bind (open "hello.txt") (λr:1:Result File IOError.
    match r with {
      Err e => println (show e)
    | Ok f  =>
        bind (read f) (λcontents:1:Result String IOError.
          bind (close f) (λ_:1:Unit.
            match contents with {
              Err e => println (show e)
            | Ok s  => println s
            }))
    })
```

Note that `close f` is called before inspecting `contents` — this is valid because
`read` borrows `f` (`&File`) while `close` consumes it (`File ->¹`). The borrow ends
before `close` is called, satisfying the borrow checker.

### 16.9 Standard Library Module Summary

```
Std
├── Prelude     -- auto-imported
│   │           -- Bool, Unit, Option, Result, Ordering
│   │           -- Eq, Ord, Hash, Clone, Copy, Drop, Sized
│   │           -- Add, Sub, Mul, Div, Rem, Neg, DivisionByZero
│   │           -- Functor, Applicative, Monad
│   │           -- not, and, or
│   │           -- all native type impls
│   │           -- Functor/Applicative/Monad for Option and Result
│   └── prim_*  -- 30 compiler intrinsics (not importable, not user-accessible)
│
├── String      -- String type and text operations           [Layer 1]
├── Show        -- Show trait and native impls               [Layer 1]
│
├── List        -- singly-linked list, map/filter/fold       [Layer 2]
├── Map         -- persistent ordered map                    [Layer 2]
├── Set         -- persistent ordered set (Map κ Unit)       [Layer 2]
│
└── IO          -- IO monad, File, IOError                   [Layer 3]
                -- impl Monad IO (orphan rule: IO defined here)
                -- impl Functor IO, impl Applicative IO
```

**Deferred to v1.1**: `Std.Array` — a contiguous growable array requiring allocator
interface design currently out of scope. `Std.Set` is a thin wrapper over `Std.Map`
and is included in scope but not fully specified here.

---

## 17. Bibliography

- Atkey, R. (2018). **Syntactic Control of Concretion**. POPL 2018.
- Girard, J.-Y. (1987). **Linear Logic**. Theoretical Computer Science.
- Lafont, Y. (1990). **Interaction Nets**. POPL 1990.
- Wadler, P. (1990). **Linear Types Can Change the World**. Programming Concepts and
  Methods.
- Sabel, D., & Schmidt-Schauß, M. (2011). **A Contextual Semantics for Concurrent
  Haskell**.

---

## 18. Appendix A: Haskell Pseudocode

*See lambda-circle-design-document-v2.2.md*

---

## 19. Appendix B: Correction Log

### v2.4 Additions and Corrections (this version)

| ID | Section | Change |
|----|---------|--------|
| E1 | §2 | Split into §2.A (concrete surface grammar, normative) and §2.B (theoretical calculus); added concrete examples |
| E2 | §2.A.1 | Removed `e.op₂ e₁` and `op₁ e` — binary/unary ops are prelude trait methods, not syntax |
| E3 | §2.A.3 | Complete type grammar with implementation status per production; added naming convention (lowercase=variable, uppercase=constructor); added `type_app` juxtaposition production; added `&` in type position; added product type; marked Phase 5a prerequisites |
| E4 | §2.A.4 | Constructor patterns noted as Phase 5a; current parser handles `_` and identifier only |
| E5 | §2.A.5 | Fully specified `sig`, `val_def`, `constraint` productions; corrected `impl C for T` ordering; noted all decls as Phase 5a/5 prerequisites |
| E6 | §2.A.6 | Expanded examples section: implemented terms, Phase 5a types, Phase 5a/5 module declarations — each group clearly labelled |
| E7 | §7.4 | Added two-phase S5′ verification rationale |
| E8 | §8.4 | Added Orphan Rule (new subsection) |
| E9 | §8.5 | Added Impl Visibility rules (new subsection) |
| E10 | §9 | Restructured: removed operations list; added §9.2 noting ops are prelude trait methods; moved Float/NaN note to §9.3 |
| E11 | §10 | Extended error table with Gate column and new module-system errors |
| E12 | §11 | Complete rewrite: one-file-one-module, visibility, import forms, six-gate pipeline, .λo format, incremental recompilation, link step |
| E13 | §12 | Added Phase 5a (grammar extensions, 2 weeks) as hard prerequisite; Phases 5 and 6 marked as blocked; added prerequisite table |
| E14 | §13 | Added resolved questions 22–31 covering grammar split, ops, div totality, boolean logic, naming convention, type application, reference types, impl syntax, Phase 5a |
| E15 | §14 | Added trade-off entries for grammar minimalism, Div totality, strict boolean ops, naming convention |
| E16 | §16.2 | Added arithmetic traits: Add, Sub, Mul, Div, Rem, Neg, DivisionByZero; added boolean prelude functions; updated native type impl matrix |
| E18 | §6.1 | Added `Prim(op)` and `PrimVal(type, value)` agents; documented typed payload, fixed arity, user-inaccessibility |
| E19 | §6.2 | Added four primitive interaction rules: binary/unary Prim⋈PrimVal, PrimVal⋈ε, PrimVal⋈δ; documented uniform evaluator loop |
| E20 | §6.3 | Renamed to Translation Policy (unchanged) |
| E21 | §6.4 | New subsection: Primitive Translation — how literals, prim_* calls, and prelude wrappers translate to Prim/PrimVal agents; end-to-end `add 3 5` example |
| E22 | §9.2 | Updated to reference §6.4 primitive agents and uniform evaluator |
| E23 | §10 | Added `UnknownPrimitive` error at Gate 5 |
| E24 | §16.1 | Updated layer hierarchy to five layers; added prim_* as compiler-internal sublayer of Prelude |
| E25 | §16.3 | New subsection: Compiler Intrinsics — complete 30-intrinsic table with arity, agent, prelude wrapper, and result type for all Int/Float/Bool/Char operations |
| E26 | §16.4–16.9 | Renumbered from 16.3–16.8 to accommodate new §16.3 |
| E27 | §16.9 | Updated module summary tree to show prim_* as non-importable sublayer of Prelude; added arithmetic traits to prelude listing |
| E28 | §6.1 | Added `PrimIO(op)` (3-port) and `IO_token` (1-port, linear) agents; documented IO_token lifecycle and sequencing semantics |
| E29 | §6.2 | Added PrimIO interaction rules; documented IO_token cannot interact with ε or δ |
| E30 | §16.2 | Added full Functor → Applicative → Monad trait hierarchy with laws; added Functor/Applicative/Monad instances for Option and Result; noted Monad IO lives in Std.IO by orphan rule |
| E31 | §16.8 | Complete rewrite: IO is a monad (`IO a` describes effectful computation); removed capability-passing design; added internal IO_token lowering explanation; monadic API with bind/then; File with independent token for concurrency; complete usage example |
| E32 | §16.9 | Updated summary tree: IO monad and Functor/Applicative/Monad in prelude; Monad IO instance in Std.IO |
| E33 | §13 | Added resolved questions 37–42: IO monad, IO location, Monad hierarchy, IO_token sequencing, do-notation deferral, File linearity |
| E34 | §13 | Added open questions: do-notation (v1.1), FFI, short-circuit ops (v2.0) |
| E35 | §14 | Added trade-off entries for monadic IO, Functor/Applicative/Monad in prelude, IO_token internal, do-notation deferral, File concurrency |
| E36 | §16.3 | Restructured into §16.3.1 (Arithmetic, 30) and §16.3.2 (IO, 9); total updated to 39; added pure/effectful distinction; added subheadings to arithmetic tables |
| E37 | §16.3.2 | New subsection: complete 9-entry IO intrinsics table (prim_io_print through prim_io_write); arity notes for nullary ReadLine and linear Close; end-to-end `println "hello"` evaluation trace |
| E38 | §6.4 | Added IO translation rules: current_token_wire threading state; translation of prim_io_println, prim_io_read_line, prim_io_close; bind desugaring to sequential token chain with concrete two-println example |
| E39 | §13 | Updated question 35 to reflect 39 intrinsics; added question 43 resolving prim_io_* wire-up |
| E40 | §14 | Updated intrinsics trade-off to reflect arithmetic/IO split and token port distinction |

### v2.3 Corrections

| ID | Section | Issue | Fix Applied |
|----|---------|-------|-------------|
| D1 | §15 | Phase 0 not documented | Added complete Lean4 formalization section |
| D2 | §13 | Open questions not resolved | Marked all as resolved |

### v2.2 Corrections

| ID | Section | Issue | Fix Applied |
|----|---------|-------|-------------|
| C1 | §4.3 | Contract rule: belongs only in net translation | Removed from source type system |
| C2 | §4.3 | Let rule: scaling corrected for q = ω case | Fixed in v2.2 definition |
| C3 | §4.3 | Borrow-Weaken rule: redundant | Removed |
| C4 | §4.2 | `0·Γ` semantics: empty vs. `_0`-annotated | Defined: `_0`-annotated |
| C5 | §4.3 | Weaken: redundant premise | Removed |
| C6 | §5.1 | View field coercion | Added explicit rule |
| C7 | §5.2 | View observability liveness | Added clause |
| C8 | §7.3 | S5' algorithm acyclicity | Added precondition |
| C9 | §9.1 | Unit description | Corrected |
| C10 | §7.5, §14 | Immutable ω consequence | Documented |
| C11 | §4.4 | Phase 0 inductive cases | Enumerated |

### v2.1 Corrections

| ID | Section | Issue | Fix Applied |
|----|---------|-------|-------------|
| B1 | §4.3 | Contract rule direction | Fixed in v2.1 |
| B2 | §2.3 | Native operations | Added to grammar |
| B3 | §2.2 | Product/sum types | Added to grammar |
| B4 | §7.3 | S5' algorithm | Specified |
| B5 | §9.3 | Hybrid execution | Documented |

---

*Document Version: 2.4*
*Created: 2026-03-27 (v2.0)*
*Revised: 2026-03-29 (v2.4)*
*Supersedes: lambda-circle-design-document-v2.3.md*
*Status: Module System & Standard Library Specified — Ready for Phase 5 & 6 Implementation*
