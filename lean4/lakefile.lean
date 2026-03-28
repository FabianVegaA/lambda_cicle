import Lake
open Lake DSL

package lambdaCalculus where
  moreLeanOptions := #[
    ⟨`interactive.fiberwiseHygiene, false⟩,
    ⟨`warningAsError false, true⟩
  ]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.16.0"
