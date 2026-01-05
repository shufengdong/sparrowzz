# SOLVER-EMBEDDED Model Examples

This repository provides small model/example suites for an external solver, organized by model type:

- [leq/](leq/): linear equation systems (Ax = b) examples
- [neq/](neq/): nonlinear equation systems f(x)=0 examples
- [nlp/](nlp/): nonlinear programming (NLP) examples (includes AC-OPF benchmarks)
- [milp/](milp/): mixed-integer linear programming (MILP) examples

Each folder contains paired `*.model` / `*.solution` files:

- `*.model` is the input model text consumed by the solver
- `*.solution` is the reference (ground-truth) result for validation

All other top-level folders are currently **work in progress (WIP)**.

## Other directories (WIP)

The following top-level directories are under active development and are not part of the stable example suites yet:

- [data/](data/)
- [index/](index/)
- [rsdss/](rsdss/)
- [rspower/](rspower/)
- [rustscript/](rustscript/)
- [target/](target/)

## CLI usage (model selection)

The solver CLI supports selecting **at most one** model input at a time:

- `./solver_embedded --nlp <FILE or String>`
- `./solver_embedded --milp <FILE or String>`
- `./solver_embedded --leq <FILE or String>`
- `./solver_embedded --neq <FILE or String>`

Inputs can be provided either as:

- a file path, or
- an inline string

### Splitting rules (common)

- If the input contains `;` -> split by `;`
- Otherwise -> split by lines

## Model formats (summary)

### NLP (`--nlp`)

- Overall:
  - `<Objective>;`
  - `<Constraint1>;`
  - `...`
  - `<VariableBounds>;` (**must be last**)
- Constraints use range form: `<expr>:[lower/upper]`
- Variables use bounds/initial value: `xi:[lower/upper/init]`

### MILP (`--milp`)

- Overall:
  - `<Objective>;`
  - `<Constraint1>;`
  - `...`
  - `<VariableTypes>;` (**must be last**)
- Constraints use boolean comparisons: `<=`, `>=`, `==`
- Variable types (last line): `x1:1, x2:2, ...`
  - `1` binary, `2` integer, `3` real/continuous

### LEQ (`--leq`)

- Overall:
  - `<Eq1>;`
  - `<Eq2>;`
  - `...`
  - `<Variables>;` (**must be last**)
- Each equation is interpreted as `<expr> = 0`
- Variables (last line): `x1, x2, x3, ...`

### NEQ (`--neq`)

- Same overall format as `--leq`, but nonlinear:
  - equations are interpreted as `<expr> = 0`
  - variables are listed on the last line

## Where to start

- LEQ toy examples: [leq/examples/basic/](leq/examples/basic/)
- NEQ toy examples: [neq/examples/basic/](neq/examples/basic/)
- NLP toy examples: [nlp/examples/basic/](nlp/examples/basic/)
- MILP toy examples: [milp/examples/basic/](milp/examples/basic/)
