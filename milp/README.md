# MILP Examples (Mixed-Integer Linear Programming)

This folder contains **MILP** benchmark instances.

The main purpose is to provide small, solver-friendly examples so an external program can validate its MILP pipeline:

- `*.model`: the MILP instance (objective + constraints + variable type declarations)
- `*.solution`: the **reference (ground-truth) solution** for that model

These examples are intended for automated regression checks and solver benchmarking. Detailed format/CLI usage is documented elsewhere.

## Directory layout

- [milp/examples/basic/](milp/examples/basic/): small MILP toy problems

## `.solution` format (reference solution)

A `*.solution` file contains one numeric value per line.

- The values are ordered **exactly as the variable list on the last line of the corresponding `.model` file**.

## How to use this

Run your solver in “nonlinear equations” mode and pass the model file, for example:

- `solver_embedded --neq path/to/case5_pf_polar.model`

Then compare your solver’s solution vector against `case5_pf_polar.solution`.

> Note: the exact CLI flag name depends on your solver/application (this repo only defines the `*.model/*.solution` formats and the examples).

