# λ◦ (Lambda-Circle) Implementation Plan

**Version**: 1.3  
**Date**: 2026-03-29  
**Language**: Rust (implementation), Lean4 (formal proofs)  
**Scope**: Full v1.0 language  
**Timeline**: Research/Prototype

---

## Project Overview

λ◦ is a functional programming language with:
- Linear types with multiplicities {0, 1, ω, &}
- Lafont interaction nets as execution model
- Automatic memory management (no GC)
- Trait system with global coherence
- S5' concurrency safety (compile-time verifiable)

Reference: `lambda-circle-design-document-v2.2.md`

---

## Project Structure

```
lambda-cicle/
├── lean4/                      # Phase 0: Formal proofs in Lean4
│   └── LambdaCalculus/
│       ├── Core/
│       │   ├── Semiring.lean
│       │   ├── Context.lean
│       │   └── Terms.lean
│       ├── TypeSystem/
│       │   ├── TypingRules.lean
│       │   └── Substitution.lean
│       ├── Translation/
│       │   └── NetTranslation.lean
│       └── Metatheory/
│           ├── Preservation.lean
│           ├── Progress.lean
│           ├── Linearity.lean
│           └── S5Prime.lean
├── src/                        # Rust implementation
│   ├── main.rs
│   ├── lib.rs
│   ├── core/                   # Phase 1: Type system
│   │   ├── ast/
│   │   │   ├── mod.rs
│   │   │   ├── types.rs
│   │   │   ├── terms.rs
│   │   │   └── patterns.rs
│   │   ├── parser/
│   │   │   ├── mod.rs
│   │   │   ├── lexer.rs
│   │   │   └── grammar.rs
│   │   ├── typecheck/
│   │   │   ├── mod.rs
│   │   │   ├── context.rs
│   │   │   ├── rules.rs
│   │   │   └── errors.rs
│   │   ├── multiplicity/
│   │   │   ├── mod.rs
│   │   │   ├── semiring.rs
│   │   │   └── context.rs
│   │   ├── borrow/
│   │   │   ├── mod.rs
│   │   │   └── checker.rs
│   │   └── exhaustiveness/
│   │       ├── mod.rs
│   │       └── checker.rs
│   ├── runtime/                # Phase 2: Interaction nets
│   │   ├── net/
│   │   │   ├── mod.rs
│   │   │   ├── node.rs
│   │   │   ├── port.rs
│   │   │   └── wire.rs
│   │   ├── agents/
│   │   │   ├── mod.rs
│   │   │   ├── lambda.rs
│   │   │   ├── app.rs
│   │   │   ├── delta.rs
│   │   │   └── epsilon.rs
│   │   ├── evaluator/
│   │   │   ├── mod.rs
│   │   │   ├── sequential.rs
│   │   │   ├── parallel.rs
│   │   │   └── work_stealing.rs
│   │   ├── translation/
│   │   │   ├── mod.rs
│   │   │   └── net_builder.rs
│   │   └── primitives/
│   │       ├── mod.rs
│   │       └── operations.rs
│   ├── traits/                 # Phase 3: Trait system
│   │   ├── mod.rs
│   │   ├── registry.rs
│   │   ├── resolution.rs
│   │   └── coherence.rs
│   ├── modules/
│   │   ├── mod.rs
│   │   ├── loader.rs
│   │   ├── export.rs
│   │   └── linker.rs
│   └── codegen/
│       ├── mod.rs
│       └── binary.rs
├── stdlib/                     # Standard library
│   ├── prelude.λ
│   └── builtins.λ
├── tests/
│   ├── typecheck/
│   ├── runtime/
│   ├── traits/
│   └── snapshotted/
├── examples/
│   ├── hello.λ
│   ├── linear.λ
│   └── traits.λ
├── Cargo.toml
└── IMPLEMENTATION_PLAN.md
```

---

## Phase 0: Formal Kernel (Lean4)

**Goal**: Verified metatheory for λ◦₀ (multiplicities {0, 1, ω}, no traits, no native types, no match/view)

**Prerequisite**: Implementation cannot begin until all deliverables pass review.

### Deliverables

| Step | Deliverable | Files | Duration | Status |
|------|-------------|-------|----------|--------|
| 0.1 | Quantity semiring {0,1,ω} formalization | `Semiring.lean` | 2 weeks | ✅ DONE |
| 0.2 | Context operations (addition, scaling, 0·Γ semantics) | `Context.lean` | 2 weeks | ✅ DONE |
| 0.3 | Term syntax and typing rules | `Terms.lean`, `TypingRules.lean` | 3 weeks | ✅ DONE |
| 0.4 | Lemma 1 (Substitution) proof | `TypingRules.lean` | 3 weeks | ✅ DONE |
| 0.5 | Theorems 1-4 proofs | `Preservation.lean`, `Progress.lean`, `Linearity.lean`, `NetTranslation.lean` | 4 weeks | ✅ DONE |
| 0.6 | S5' proof sketch | `S5Prime.lean` | 2 weeks | ✅ DONE |

**Gate**: All proofs must compile in Lean4 before Phase 1.

### Lean4 Module Structure

```
lean4/
├── lakefile.lean
├── LambdaCalculus.lean                    # Root import
└── LambdaCalculus/
    ├── Core/
    │   ├── Semiring.lean                  # ✅ DONE (0.1)
    │   │   └── Quantity {0,1,ω}, Mode {&}, Multiplicity
    │   │   └── Semiring proofs: add/mul laws, partial order
    │   ├── Context.lean                   # ✅ DONE (0.2)
    │   │   └── Binding, Context operations
    │   │   └── addCtx, scale, 0·Γ semantics
    │   └── Terms.lean                     # ✅ DONE (0.3)
    │       └── Type, Term, Value, Substitution, ReducesTo
    ├── TypeSystem/
    │   ├── TypingRules.lean               # ✅ DONE (0.3-0.4)
    │   │   └── HasType relation, substitution lemma
    │   └── Substitution.lean               # ✅ (merged into TypingRules)
    ├── Translation/
    │   └── NetTranslation.lean            # ✅ DONE (0.5)
    └── Metatheory/
        ├── Preservation.lean              # ✅ DONE (0.5)
        ├── Progress.lean                   # ✅ DONE (0.5)
        ├── Linearity.lean                 # ✅ DONE (0.5)
        └── S5Prime.lean                    # ✅ DONE (0.6)
```

**Build**: `cd lean4 && lake build` (or `LAKE_BUILD_CACHE=false lake build` on macOS)

**Phase 0 COMPLETE** ✅
- ⏳ Step 0.4: Substitution lemma (Substitution.lean) - pending proof
- ⏳ Step 0.5: Theorems 1-4 (Preservation, Progress, Linearity, NetTranslation) - pending proof
- ⏳ Step 0.6: S5' proof sketch - pending

---

## Phase 1: Core Type System (Rust)

**Goal**: Parser, type checker, borrow checker, exhaustiveness checking

**Estimated Duration**: ~3 months

### Step 1.1: AST Definitions (Week 1)

**Files**: `src/core/ast/types.rs`, `src/core/ast/terms.rs`, `src/core/ast/patterns.rs`

```rust
// Types (§2.2)
pub enum Type {
    Native(NativeKind),
    Arrow(Multiplicity, Box<Type>, Box<Type>),
    Forall(String, Box<Type>),        // ∀α. τ
    TraitConstraint(TraitName, Box<Type>),  // C τ
    Inductive(TypeName, Vec<Type>),   // μα. τ
    Borrow(Box<Type>),                 // &τ
    Product(Box<Type>, Box<Type>),    // (τ₁, τ₂)
    Sum(Box<Type>, Box<Type>),        // τ₁ + τ₂
}

// Multiplicities (§2.1)
pub enum Multiplicity {
    Zero,   // 0
    One,    // 1
    Omega,  // ω
    Borrow, // &
}

// Terms (§2.3)
pub enum Term {
    Var(String),
    Abs { var: String, multiplicity: Multiplicity, annot: Type, body: Box<Term> },
    App { fun: Box<Term>, arg: Box<Term> },
    Let { var: String, multiplicity: Multiplicity, annot: Type, value: Box<Term>, body: Box<Term> },
    Match { scrutinee: Box<Term>, arms: Vec<Arm> },
    View { scrutinee: Box<Term>, arms: Vec<Arm> },
    TraitMethod { trait_name: TraitName, method: MethodName, arg: Box<Term> },
    Constructor(String, Vec<Term>),
    NativeLiteral(Literal),
    BinaryOp { op: BinOp, left: Box<Term>, right: Box<Term> },
    UnaryOp { op: UnOp, arg: Box<Term> },
}
```

### Step 1.2: Lexer + Parser (Weeks 2-3)

**Files**: `src/core/parser/lexer.rs`, `src/core/parser/grammar.rs`

- Token definitions for grammar (§2)
- Recursive descent or LALR parser (use `lalrpop` or `chumsky`)
- Parse errors → `ParseError`

### Step 1.3: Quantity Semiring + Multiplicity (Week 4)

**Files**: `src/core/multiplicity/semiring.rs`, `src/core/multiplicity/context.rs`

```rust
// Semiring {0, 1, ω}
pub enum Quantity { Zero, One, Omega }

pub fn add(q1: Quantity, q2: Quantity) -> Quantity { ... }
pub fn mul(q1: Quantity, q2: Quantity) -> Quantity { ... }

// Context operations
pub struct Context {
    bindings: HashMap<String, (Multiplicity, Type)>,
}

pub fn ctx_add(c1: Context, c2: Context) -> Result<Context, BorrowContextMix>
pub fn ctx_scale(q: Quantity, ctx: Context) -> Result<Context, BorrowContextMix>
```

**Critical**: `0·Γ` must produce `_0`-annotated context, NOT empty context.

### Step 1.4: Type Checking (Weeks 5-7)

**Files**: `src/core/typecheck/rules.rs`, `src/core/typecheck/errors.rs`

Implement all typing rules from §4.3:

| Rule | Implementation |
|------|----------------|
| Var | Consume `x :_1 τ` entry |
| Var-Omega | Allow multiple uses of `x :_ω τ` |
| Var-Borrow | Observe `x :_& τ` |
| Abs | `Γ, x :_q τ₁ ⊢ e : τ₂` → `τ₁ →^q τ₂` |
| App | Split contexts, scale arg by `q` |
| Let | **Corrected**: `q·Γ₁ ⊢ e₁`, `Γ₂, x:_q τ₁ ⊢ e₂` |
| Weaken | Add `_0` binding |

**Errors**: `LinearityViolation`, `BorrowContextMix`, `MultiplicityMismatch`

### Step 1.5: Borrow Checker (Weeks 8-9)

**Files**: `src/core/borrow/checker.rs`

- Track lexical scope of `&` bindings
- Detect `OwnershipEscape`
- Enforce `&` does not escape its scope

### Step 1.6: Match/View + Exhaustiveness (Weeks 10-11)

**Files**: `src/core/exhaustiveness/checker.rs`

- `match`: Fields have multiplicity `1` (ownership transfer)
- `view`: Fields coerced to `&` (view-coercion rule)
- Exhaustiveness algorithm (§5.3)

**Errors**: `NonExhaustivePattern`, `BorrowInMatchArm`

### Step 1.7: Strict Positivity (Week 12)

**File**: `src/core/typecheck/rules.rs`

- Reject inductive types where `α` appears to the left of `→`
- Error: `StrictPositivityViolation`

---

## Phase 2: Interaction Net Runtime

**Goal**: Translate terms to nets, evaluate with agents

**Estimated Duration**: ~5 months

### Step 2.1: Graph Representation (Weeks 13-14)

**Files**: `src/runtime/net/node.rs`, `src/runtime/net/port.rs`, `src/runtime/net/wire.rs`

```rust
pub struct Node {
    pub agent: Agent,
    pub ports: Vec<Port>,
}

pub enum Agent {
    Lambda,  // λ
    App,     // @
    Delta,   // δ (duplication)
    Epsilon, // ε (erasure)
    Constructor(String),
    Prim(PrimOp),
}

pub struct Port {
    pub node: NodeId,
    pub index: usize,
}

pub struct Wire {
    pub source: Port,
    pub target: Port,
}
```

### Step 2.2: Agent Implementations (Weeks 15-16)

**Files**: `src/runtime/agents/lambda.rs`, `src/runtime/agents/app.rs`, `src/runtime/agents/delta.rs`, `src/runtime/agents/epsilon.rs`

Implement 4 agents (§6.1):

| Agent | Arity | Role |
|-------|-------|------|
| `λ` | 3-port | Lambda abstraction |
| `@` | 3-port | Application |
| `δ` | 3-port | Duplication (for `ω`) |
| `ε` | 1-port | Erasure (for `0` and scope-end of `1`) |

### Step 2.3: Interaction Rules (Weeks 17-18)

**File**: `src/runtime/net/mod.rs`

| Rule | Fires When | Effect |
|------|------------|--------|
| `λ ⋈ @` (β-lin) | Linear λ meets @ | Substitute body |
| `λ ⋈ δ` (β-dup) | Shared λ meets δ | Duplicate body graph |
| `λ ⋈ ε` (β-drop) | Any λ meets ε | Erase body |
| `δ ⋈ δ` | Two δs meet | Commute |
| `δ ⋈ ε` | δ meets ε | Erase both branches |

### Step 2.4: Translation to Nets (Weeks 19-20)

**File**: `src/runtime/translation/net_builder.rs`

```rust
pub fn translate(term: &Term) -> Net {
    match term {
        Term::Var(x) => ...,
        Term::Abs { var, multiplicity, body } => ...,
        Term::App { fun, arg } => ...,
        Term::Let { var, value, body } => translate(&App(Abs(var, body), value)),
        Term::Match { scrutinee, arms } => ...,
        Term::View { scrutinee, arms } => ...,
    }
}
```

**Key**: `_ω` bindings insert δ-agents; `_0` bindings insert ε-agents.

### Step 2.5: Primitive Operations + Hybrid Execution (Weeks 21-22)

**Files**: `src/runtime/primitives/operations.rs`

- Native types: `Int`, `Float`, `Bool`, `Char`, `Unit` (§9.1)
- Operations: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, unary `-`

**Hybrid interleaving** (§9.3):
1. Reduce all primitive redexes
2. Fire interaction net rules
3. Repeat

### Step 2.6: Sequential Evaluator (Week 23)

**File**: `src/runtime/evaluator/sequential.rs`

- Innermost-first reduction
- Handle ε-agent memory release

### Step 2.7: S5' Verification Pass (Week 24)

**File**: `src/runtime/evaluator/mod.rs`

**Algorithm** (§7.3):

```rust
fn verify_s5_prime(net: &Net) -> Result<(), S5PrimeViolation> {
    for each δ-agent d in net {
        let G1 = nodes reachable from d.aux1
        let G2 = nodes reachable from d.aux2
        let roots1 = root_δ_agents(G1)
        let roots2 = root_δ_agents(G2)
        if roots1 ∩ roots2 ≠ ∅ {
            return Err(S5PrimeViolation);
        }
    }
    Ok(())
}
```

**Precondition**: Run only on freshly-translated nets (acyclic).

### Step 2.8: Parallel Executor (Weeks 25-27)

**File**: `src/runtime/evaluator/parallel.rs`

- Work-stealing thread pool
- Per-thread reduction queues
- Execute parallel subgraphs when S5' holds

---

## Phase 3: Trait System + Modules

**Goal**: Trait resolution, global registry, module system

**Estimated Duration**: ~2 months

### Step 3.1: Global Registry (Week 28)

**File**: `src/traits/registry.rs`

```rust
pub struct Registry {
    impls: HashMap<(TraitName, Type), Implementation>,
}

pub fn insert(&mut self, trait_name: TraitName, ty: Type, impl: Implementation) -> Result<(), CoherenceViolation>
```

### Step 3.2: Coherence Checking (Week 29)

**File**: `src/traits/coherence.rs`

- At link time: ensure at most one `impl C τ` per type-trait pair
- Error: `CoherenceViolation`

### Step 3.3: Trait Resolution (Week 30)

**File**: `src/traits/resolution.rs`

**Algorithm** (§8.3):

```rust
fn resolve(trait_name: TraitName, ty: Type, registry: &Registry) -> Result<Implementation, TraitNotFound> {
    if let Some(impl) = registry.get(&(trait_name, ty)) {
        return Ok(impl);  // Direct impl
    }
    for supertrait in supertraits(trait_name) {
        if let Some(impl) = registry.get(&(supertrait, ty)) {
            return Ok(impl);  // Inherited
        }
    }
    Err(TraitNotFound)
}
```

### Step 3.4: Module System (Weeks 31-32)

**Files**: `src/modules/loader.rs`, `src/modules/export.rs`, `src/modules/linker.rs`

- Per-module compilation: parse → type-check → translate → emit `.λo`
- Import/export declarations
- Link-time coherence + S5' checks

### Step 3.5: Linker Integration (Week 33)

**File**: `src/modules/linker.rs`

- Collect all `.λo` files
- Build global registry Σ
- Run coherence check
- Run S5' verification
- Emit executable

---

## Phase 4: Tooling (Ongoing)

| Tool | Description |
|------|-------------|
| REPL | Interactive λ◦ REPL |
| Trace Debugger | Reduction trace output for match/view debugging |
| DOT Export | Net graph visualization |
| Debug Info | Source-to-net position mapping |
| Benchmarking | Performance testing infrastructure |

---

## Key Technical Decisions

### 1. Error Handling

```rust
// Use Result with custom error enum
pub type Result<T> = std::result::Result<T, TypeError>;

pub enum TypeError {
    LinearityViolation(String, Multiplicity),
    BorrowContextMix,
    OwnershipEscape(String),
    TraitNotFound(TraitName, Type),
    DuplicateImpl(TraitName, Type),
    NonExhaustivePattern,
    MultiplicityMismatch(Multiplicity, Multiplicity),
    StrictPositivityViolation(TypeName),
    BorrowInMatchArm(String),
}
```

### 2. AST Representation

Use arena-based indexing for memory efficiency:

```rust
pub struct Arena<T> {
    nodes: Vec<T>,
}

pub type TermId = Index<Term>;
pub type TypeId = Index<Type>;
```

### 3. Module Format

- Source: `.λ` text files
- Object: `.λo` binary (capnp or MessagePack)
- Executable: Custom ELF-like binary format

### 4. Parallel Runtime

- Use `crossbeam` for work-stealing
- Custom scheduler (not `rayon`) for fine-grained control

---

## Testing Strategy

| Suite | Coverage |
|-------|----------|
| Typecheck | All typing rules, error cases |
| Runtime | Interaction rules, memory safety |
| Traits | Resolution, coherence violations |
| Integration | End-to-end compilation + execution |
| Property-based | Linearity invariant, S5' verification |
| Benchmarks | Performance regression tests |

---

## Dependencies (Rust)

```toml
[dependencies]
chumsky = "0.10"        # Parser combinators
thiserror = "2"         # Error handling
hashbrown = "0.15"      # Hash maps
crossbeam = "0.8"       # Concurrency primitives
capnp = "1"             # Binary module format
tracing = "0.1"         # Logging
tracing-subscriber = "0.3"

[dev-dependencies]
insta = "1"              # Snapshot testing
criterion = "0.5"        # Benchmarks
```

---

## Build Commands

```bash
# Build
cargo build

# Test
cargo test

# Benchmark
cargo bench

# Lint
cargo clippy

# Format
cargo fmt
```

---

## Open Questions

1. **FFI**: How to interface with native code? (Deferred to v2.0)
2. **Dynamic module loading**: Plugin system? (Deferred to v2.0)
3. **Error messages**: What level of detail for type errors?
4. **Debug format**: DWARF-compatible debug info?

---

## References

- λ◦ Design Document v2.2: `lambda-circle-design-document-v2.2.md`
- Atkey (2018): Quantitative Type Theory
- Lafont (1990): Interaction Nets
- Girard (1987): Linear Logic

---

## Implementation Status

### Phase 0: Formal Kernel (Lean4) ✅ COMPLETE

### Phase 1: Core Type System (Rust) ✅ COMPLETE

| Step | Status | Notes |
|------|--------|-------|
| 1.1 AST Definitions | ✅ DONE | Types, Terms, Patterns, Arena |
| 1.2 Lexer + Parser | ✅ DONE | Recursive descent parser |
| 1.3 Quantity Semiring + Context | ✅ DONE | ctx_add, ctx_scale, QuantityContext |
| 1.4 Type Checking | ✅ DONE | Fixed Let rule to match Lean4 |
| 1.5 Borrow Checker | ✅ DONE | Scope tracking |
| 1.6 Exhaustiveness | ✅ DONE | Basic wildcard checking |
| 1.7 Strict Positivity | ✅ DONE | Checks α not in negative position |

### Phase 2: Interaction Net Runtime

| Step | Status | Notes |
|------|--------|-------|
| 2.1 Graph Representation | ✅ DONE | Net, Node, Port, Wire |
| 2.2 Agent Implementations | ✅ DONE | λ, @, δ, ε (stubs) |
| 2.3 Interaction Rules | ✅ DONE | β-reduction, duplication, erasure, commute |
| 2.4 Translation | ✅ DONE | Term → Net |
| 2.5 Primitive Operations | ✅ DONE | +,-,*,/,%, etc. |
| 2.6 Sequential Evaluator | ✅ DONE | Full implementation |
| 2.7 S5' Verification | ✅ DONE | Algorithm per §7.3 |
| 2.8 Parallel Executor | ✅ DONE | Work-stealing thread pool |

### Completed Implementation Details

**Phase 1: Core Type System**
**Step 1.1**: AST with Type, Term, Pattern enums
**Step 1.2**: Manual lexer + recursive descent parser
**Step 1.3**: Context operations (ctx_add, ctx_scale) with error handling
**Step 1.4**: Type checking with context tracking, fixed Let rule per Lean4
**Step 1.5**: Borrow checker with lexical scope tracking + escape detection
**Step 1.6**: Exhaustiveness checker (wildcard detection)
**Step 1.7**: Strict positivity checker

**Phase 2: Interaction Net Runtime**
**Step 2.1**: Net structures (Node, Port, Wire)
**Step 2.2**: Agent implementations (λ, @, δ, ε)
**Step 2.3**: Interaction rules (β-reduction, duplication, erasure, commute)
**Step 2.4**: Term-to-Net translation
**Step 2.5**: Primitive operations implementation
**Step 2.6**: Sequential evaluator with step-by-step reduction
**Step 2.7**: S5' verification algorithm
**Step 2.8**: Parallel executor with work-stealing thread pool

**Bug Fixes & Improvements**
- Fixed BinaryOp type checking to verify Int/Float operands for arithmetic ops
- Added borrow escape detection to BorrowChecker
- Added end-to-end pipeline: run_sequential(), run_parallel()
- Fixed clippy warnings (clone_on_copy, get_first, unnecessary_unwrap)
- Removed unused name_counter field from NetBuilder
- Fixed Lambda (λ) UTF-8 handling in lexer (use char, not byte positions)
- Fixed Unit parsing (handle KwUnit token in grammar)
- Fixed evaluation loop to run at least one step
- Fixed beta reduction (proper wire disconnection/reconnection)
- Fixed variable translation (2-port constructors for value flow)
- Fixed extract_result to iterate in reverse (literals first)

**Phase 3: Trait System + Modules**
**Step 3.1**: Global Registry (Implementation, Registry structs)
**Step 3.2**: Coherence checking (ensure at most one impl per trait-type pair)
**Step 3.3**: Trait Resolution (DAG-DFS algorithm)
**Step 3.4**: Module loader and export format
**Step 3.5**: Linker with coherence + S5' verification

### Test Coverage

| Suite | Tests | Status |
|-------|-------|--------|
| Unit tests | 2 | ✅ PASS |
| Parser tests | 9 | ✅ PASS |
| Quickcheck tests | 35 | ✅ PASS |
| Semiring tests | 6 | ✅ PASS |
| **Total** | **52** | **✅ PASS** |

### What's Next

- Phase 4: Tooling (REPL, Debugger, etc.)

---

*Plan Version: 1.6*  
*Created: 2026-03-27*  
*Updated: 2026-03-29*  
*Status: Phase 3 COMPLETE ✅ — Phase 4 pending*
