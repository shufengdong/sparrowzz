# NLP Examples (AC Optimal Power Flow)

This folder provides **AC Optimal Power Flow (AC-OPF)** benchmark instances written as **Nonlinear Programming (NLP)** models in two coordinate systems:

- **Polar**: voltage state variables are $V$ and $\theta$.
- **Rectangular**: voltage state variables are $e$ and $f$ (i.e., $V\cos\theta$ and $V\sin\theta$).

The underlying cases come from **IEEE test systems / MATPOWER**. This repo already ships ready-to-use `*.model/*.solution` examples. **MATLAB** (with MATPOWER; IPOPT used by MATPOWER OPF) is only needed if you want to regenerate or add new examples.

The purpose of these examples is:

- `*.model`: input for an **external solver** (e.g., via a `--nlp` flag)
- `*.solution`: the **reference (ground-truth) solution** for that model, used to validate your solver

## What the examples mean (main point)

Each example is a reproducible OPF-NLP benchmark:

- **Input**: a `*.model` file
- **Reference output**: a matching `*.solution` file

Your solver reads the `.model`, solves it, then you compare your solver’s objective/variables against the `.solution`.

## `.model` (model file) and variable definitions

`*.model` is a plain-text NLP description:

1. **Line 1**: objective function (typically a polynomial cost in generator active powers `PGk`)
2. **Last line**: variable definitions with bounds and an initial value, written like: `PG1:[lb/ub/x0]`

Variables:

- Polar: `PGk, QGk, Vi, THETAi`
  - `Vi`: bus voltage magnitude
  - `THETAi`: bus voltage angle (radians)
- Rectangular: `PGk, QGk, ei, fi`
  - `ei = Vi*cos(THETAi)`
  - `fi = Vi*sin(THETAi)`

## `.solution` (reference solution)

`*.solution` is the reference answer for the corresponding `.model`:

1. Line 1: optimal objective value
2. Remaining lines: optimal variable values, written one value per line, in a fixed order (consistent with the generator that produced the files)

## Directory layout

- [nlp/examples/](nlp/examples/)
  - `opf_polar/`: paired polar OPF `*.model/*.solution`
  - `opf_rectangular/`: paired rectangular OPF `*.model/*.solution`
  - `basic/`: small, solver-friendly NLP toy problems (no power-system context)

## Naming convention

- Polar
  - `opf_polar/<case>_opf_polar.model`
  - `opf_polar/<case>_opf_polar.solution`
- Rectangular
  - `opf_rectangular/<case>_opf_rectangular.model`
  - `opf_rectangular/<case>_opf_rectangular.solution`

- Basic NLP
  - `basic/<name>.model`
  - `basic/<name>.solution`

## How to use this

Run your solver in “nonlinear equations” mode and pass the model file, for example:

- `solver_embedded --neq path/to/case5_pf_polar.model`

Then compare your solver’s solution vector against `case5_pf_polar.solution`.

> Note: the exact CLI flag name depends on your solver/application (this repo only defines the `*.model/*.solution` formats and the examples).

