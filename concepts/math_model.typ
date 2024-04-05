#set page(
  paper: "us-letter",
  numbering: "1",
  header: align(right)[
    Formal Verification Model in Folidity (_draft_)
  ],
)
#set par(justify: true,)
#set text(
  font: "New Computer Modern",
  size: 12pt,
)

#align(center, text(17pt)[
  #v(1em)
  *Formal Verification Model \
  in Folidity* (_draft_)
])

#show heading: it => [
  #it.body
  #v(0.5em)
]

= Introduction

Folidity features model-first approach in coding and design of smart contracts. This can be faciliated by numerous techniques that have been actively researched in the fields of symbolic execution, model-bounded verification, static analysis, etc.

As a good starting point, it was decided to opt-in for symbolic execution, static analysis and bounded model checking.

= Verification techniques

A typical program in folidity consists of data structures that describe models, states, and function that can interact with each other. Models are one of core structures that provide the model consistency guarantee in folidity. States can encapsulte different of the same models and describe explicit state transition or state mutations as part of program execution, and functions are the driving points in program execution. Functions declares and describe the state transitions.

One of the core pieces in the workflow aforementioned is the model bounds that consist of individual boolean constraints. As shown below:

```
# Some model and its constraints
model ModelA {
  x: int,
  y: int
} st [
  x > 10,
  y < 5
]
# A state that encapsulates a model and provides its own constraints.
state StateA(ModelA) st [
  x < y
]
# A function that describes mutation.
fn () mutate(value: int) when (StateA s) -> StateA
st [
    value > 100,
    value < 100,
] { ... }

```

Let's break down how each of the selected techniques can be applied to the program written in Folidity.


As a good starting point, we can perform a static analysis and verify that the program statements, declarations and constraints are valid and consistent.

A a simple approach is to perform semantic analysis that carries out type checking and verfication of correct state transition in the function body. Specifically, if `mutate()` expect to return `StateA`, but instead it perform a state transition to `StateB` we can already detect that at a compile time.

The next stage of the analysis involves verification of the consistency of models described.

We can generalise the approach using the following mathematical model.

We can describe some verification system $bold(italic("VS"))$ as 
$bold(italic("VS")) = angle.l bold(Mu), bold(Epsilon), bold(Upsilon), Theta, Tau_Mu, Tau_(Epsilon, {Epsilon, Mu}), Tau_(Upsilon, Epsilon) angle.r$ where
- $bold(Mu)$ - set of models in the system.
- $bold(Epsilon)$ - set of states in the system
- $bold(Upsilon)$ - set of functions in the system.
- $Theta$ - set of of constraint blocks in the system, where $Theta[bold(Mu)]$ corresponds to the set of constraints for models, $Theta[bold(Epsilon)]$ - state constraints and $Theta[bold(Upsilon)]$ function constraints.
- $Tau_Mu$ - a relation $Tau: bold(Mu) harpoon.rt bold(Mu)$ describing a model inheritance. 
- $Tau_(Epsilon, {Epsilon, Mu})$ - a relation $Tau: bold(Epsilon) harpoon.rt {bold(Epsilon), bold(Mu)}$ describing any state transition bounds and encapsulated models in states, that is some state `S'` can only be transitioned to from the specified state `S`, and state some state `S` can encapsulate some model `M`
- $Tau_(Upsilon, Epsilon)$ - a relation $Tau: bold(Upsilon) harpoon.rt bold(Epsilon)$ describing any state transition bounds for states $bold(Epsilon)$ in functions $bold(Upsilon)$

In particular, $forall mu in bold(Mu) " " exists theta in Theta[mu]$ where $theta$ is a set of constraints for $mu$, and corresponding logic can be applied for elements of $Epsilon$ and $Upsilon$.

Then, to verify consistency of the system, we first need to verify the following satisfability _Sat_:

$ forall mu in bold(Mu) \
exists theta in Theta[mu] \
"s.t." theta = {c_0, c_1, ..., c_k} \
(and.big_(i) c_i) => italic("Sat") $ 
#pagebreak()
We can define the following check by some functions $rho(theta): Theta -> {italic("Sat"), italic("Unsat")}$

which yields the following proof:
$ 
exists theta in Theta[e]  \
"s.t." theta = {c_0, c_1, ..., c_k} \
(and.big_(i) c_i) => italic("Sat or Unsat") 
$ 

This allows to validate the next property of $bold(italic("VS"))$
$ 
A = { bold(Mu) union bold(Epsilon) union Upsilon } \
A = { e_0, e_1, ..., e_k } \
(and.big_(i) rho(Theta[e_i]) )=> italic("Sat or Unsat") 
$ 

The next stage is to verify co-dependent symbols in the system for satisfability of their respective constraints.

Let's look at the models $bold(Mu)$, we want to ensure that
$
"if for some" m in Mu, m' in Mu \
exists (m, m') in Tau_Mu \
"s.t." rho(m) times rho(m') = (italic("Sat"), italic("Sat")) \
"and" theta = Theta[m] union Theta[m'] \
rho(theta) => italic("Sat")
$

Very similar verification can applied to $Tau_(Upsilon, Epsilon)$.

For $Tau_(Epsilon, {Epsilon, Mu})$, the constraints can be extracted in the following way:

$
"if for some" epsilon in Epsilon, epsilon' in Epsilon \
exists (epsilon, epsilon') in Tau_(Epsilon, {Epsilon, Mu}) \
"s.t." rho(epsilon) times rho(epsilon') times rho(mu) = (italic("Sat"), italic("Sat")) \
"and" theta = Theta[epsilon] union Theta[epsilon'] \
rho(theta) => italic("Sat")
$

#pagebreak()
Similarly
$
"if for some" epsilon in Epsilon, mu in Mu \
exists (epsilon, mu) in Tau_(Epsilon, {Epsilon, Mu}) \
"s.t." rho(epsilon) times rho(mu) = (italic("Sat"), italic("Sat")) \
"and" theta = Theta[epsilon] union Theta[mu] \
rho(theta) => italic("Sat")
$

After the completing verification of `Tau` relations for consistency, we can provide a mathematical guarantee that *_VS_* has been modelled consistently.

Having verified the constraints, we can leverage them as the guards during state transions and can apply proofs from _temporal logic_ to verify that the described state transitions will take place under the described constraints.

As the final stage, we can perform the symbolic execution of instructions in the function bodies with the constraints loaded in the global context of the system. Having tracked the states of different symbols, we can verify each function for reachability for described state transitions and provide strong guanratees of functional correctness of the system described in the smart contract.