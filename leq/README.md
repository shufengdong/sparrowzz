# LEQ Examples (Linear Equations)

This folder contains **linear-equation (LEQ)** toy benchmarks.

The purpose is to provide very small, solver-friendly instances so an external program can quickly validate:

- parsing of `*.model`
- solving a linear system
- matching the reference `*.solution`

## What the examples mean

Each example is a reproducible linear-equations benchmark:

- **Input**: a `*.model` file describing a system of equations (typically linear)
- **Reference output**: a matching `*.solution` file containing a known-correct solution vector

## `.model` format

A `*.model` file is plain text:

- one equation per line, terminated by `;`
- the last line is the **variable list** (comma-separated)
  - optional initial guesses can be provided using `name:value` (e.g., `x:0`)

The variable list order defines the order of values in the `.solution` file.

## `.solution` format

A `*.solution` file contains one numeric value per line, ordered exactly as the variable list in the corresponding `.model`.

## Directory layout

- [leq/examples/basic/](leq/examples/basic/): simple LEQ toy problems

## How to use this

Run your solver in “nonlinear equations” mode and pass the model file, for example:

- `solver_embedded --neq path/to/case5_pf_polar.model`

Then compare your solver’s solution vector against `case5_pf_polar.solution`.

> Note: the exact CLI flag name depends on your solver/application (this repo only defines the `*.model/*.solution` formats and the examples).


