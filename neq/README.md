
# NEQ Examples (AC Power Flow as Nonlinear Equations)

This folder contains **nonlinear-equation (NEQ)** benchmark instances for **AC power flow (PF)**.

The underlying test cases are based on **IEEE/MATPOWER** networks. The repository ships ready-to-use examples; you typically only need to point your external solver at a `*.model` file and validate against the matching `*.solution`.

## What the examples mean (main point)

Each example is a reproducible PF-NEQ benchmark:

- **Input**: a `*.model` file describing a system of nonlinear equations $F(x)=0$
- **Reference output**: a matching `*.solution` file containing a known-correct solution vector for that model

Your external solver reads the `.model`, solves the nonlinear system, then compares its solution against the `.solution`.

## Directory layout

- [neq/examples/](neq/examples/)
	- `pf_polar/`: power-flow equations in **polar** voltage coordinates
	- `basic/`: small, solver-friendly NEQ toy problems (no power-system context)

## Naming convention

- Polar PF
	- `pf_polar/<case>_pf_polar.model`
	- `pf_polar/<case>_pf_polar.solution`

- Basic NEQ
	- `basic/<name>.model`
	- `basic/<name>.solution`

## `.model` format (NEQ model)

A `*.model` file is plain text and typically contains:

- A list of equations, one per line, each terminated by `;`
- A final **variable list line**, where variables are comma-separated

### Variable list and initial guesses

The final line lists the variables (and optional initial guesses) in the exact order used by the solution vector.

- Variables may optionally carry an initial guess using `name:value` (e.g., `V1:1`)
- Variables without `:value` have no explicit initial guess in the file

Example pattern:

- `V1:1,V2:1,...,THETA1,THETA2,...,QG1,QG2,...,PG_balancenode`

## `.solution` format (reference solution)

A `*.solution` file contains one numeric value per line.

- The values are ordered **exactly as the variable list on the last line of the corresponding `.model` file**.

## How to use this

Run your solver in “nonlinear equations” mode and pass the model file, for example:

- `solver_embedded --neq path/to/case5_pf_polar.model`

Then compare your solver’s solution vector against `case5_pf_polar.solution`.

> Note: the exact CLI flag name depends on your solver/application (this repo only defines the `*.model/*.solution` formats and the examples).
