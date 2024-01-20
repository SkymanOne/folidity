#import "ecsproject.typ": *
#import "@preview/tablex:0.0.8": tablex, rowspanx, colspanx, gridx, hlinex
#import "@preview/i-figured:0.2.3"


#let abstract = "This paper addresses the long-lasting problem involving the exploits of Smart Contract vulnerabilities. There are tools, such as in the formal verificatio field and alternative Smart Contract languages, that attempt to address these issues. However, neither approach has managed to combine the static formal verification and the generation of runtime assertions. Furthermore, this work believes that implicit hidden state transition is the root cause of securit compromises. In light of the above, we introduce Folidity, a safe functional Smart Contract language with a unique approach to reasoning about the modelling and development of Smart Contract systems. Folidity features explicit state transition checks, a model-first approach, and built-in formal verification tooling."

#let scv(code) = [
  #text(style: "italic", "SCV" + str(code)) #label("SCV:" + str(code))
]

#let ref_scv(code) = {
 link(label("SCV:" + str(code)))[_SCV#str(code)_] 
}

#let ref_req(code) = {
  link(label("Requirement:" + str(code)))[_Requirement #str(code)_]
}

#show figure: i-figured.show-figure
#show heading: i-figured.reset-counters

#show: doc => use_project(
  title: "Folidity - Safe Functional Smart Contract Language",
  author: (
    name: "German Nikolishin",
    email: "gn2g21@soton.ac.uk"
  ),
  supervisor: (
    name: "Prof. Vladimiro Sassone",
    email: "vsassone@soton.ac.uk"
  ),
  examiner: (
    name: "Dr. Indu Bodala",
    email: "i.p.bodala@soton.ac.uk"
  ),
  date: "December 12, 2023",
  program: "BSc Computer Science",
  is_progress_report: true,
  abstract_text: abstract,
  acknowledgments_text: none,
  doc
)


= Introduction <test>

The concept of "smart contract" (SC) was first coined by Nick Szabo as a computerised transaction protocol @nz_sc.
He later defined smart contracts as observable, verifiable, privity-applicable, and enforceable programs @nz_sc_bb.
In other words, smart contracts were envisioned to inherit the natural properties of traditional "paper-based" contracts.

In 2014 SCs were technically formalised at the protocol level by Dr. Gavin Wood as an arbitrary program
written in some programming language (Solidity) and executed in the blockchain's virtual machine of Ethereum @eth_yellow_paper.

Ethereum Virtual Machine (EVM) iterated over the idea of Bitcoin Scripting @bitcoin, allowing developers to deploy general-purpose, Turing-Complete
programs that can have their own storage, hence the state, written in Solidity @solidity_docs. This enabled sophisticated applications that grew beyond
the simple funds transfers among users.

Overall, SC can be summarised as an _immutable_, _permissionless_, _deterministic_ computer program
that is executed as part of state transition in the blockchain system @hardvard_sc@eth_yellow_paper. 

After a relatively short time, SCs have come a long way and allowed users to access different online services, also known as Decentralised Applications (DApps), in a completely trustless and decentralised way.
The applications have spanned financial, health, construction @cad_blockchain, and other sectors.

#pagebreak()

= Security and Safety of Smart Contracts

== Overview

With the increased adoption of DApps and total value locked in them, 
there have been numerous attacks focused on extracting funds from SCs. 
Due to the permissionless nature of SCs, the most common attack vector exploits the mistakes in the SC's source code.
Specifically, the attacker can not tamper with the protocol code due to consensus mechanisms.
Instead, they can interact with the publicly accessible parameters and interfaces to force the SC into an unexpected state, essentially gaining partial control of it.

A notorious example is the DAO hack when hackers exploited unprotected re-entrance calls to withdraw *\$50 million worth of ETH*. 
This event forced the community to hard-fork the protocol to revert the transaction, provoking a debate on the soundness of the action @the_dao.

Another less-known example is the "King of the Ether" attack, which was caused by the unchecked low-level Solidity `send` call to transfer funds to a contract-based wallet @king_of_the_ether.
The "King of the Ether Throne" contract could not recognise the failed transaction on the wallet side. Instead, the contract proceeded with the operation, incorrectly mutating its internal state. 

Other issues involve the _safety_ and _liveness_ of SCs. 
The term _safety_ is used to describe the _functional safety_ and _type safety_. _Functional safety_ refers to the guarantees that the system behaves according to the specification irrespective of the input data @func_safety,
whereas _type safety_ refers to the guarantees that the language provides a sound type system @types_pierce.
The two are often used interchangeably with the _security_ of code
as compromising the former affects the latter. When talking about _liveness_, we describe the business logic of a DApp, particularly whether it transitions into the expected new state @liveness_rob.
This is particularly important for the execution of mission-critical software in a distributed context.

_Safety_ and _liveness_ can be compromised due to the programmer's mistakes in the source code that can result in the SC entering the terminal in an unexpected state
preventing users from interacting with it @ondo_report.

== Vulnerability classification

There has been an effort in both academia and industry to classify common vulnerabilities 
and exploits in SCs in blockchain systems @owasp @stefano @atzei_survey. 
Some of the work has been recycled by bug bounty platforms, growing the community of auditors
and encouraging peer-review of SCs through the websites such as _Code4rena_#footnote[https://code4rena.com], _Solodit_#footnote[https://solodit.xyz],
and many others.

Analysing the work mentioned above, SC vulnerabilities can be categorised into the six general groups outlined in @fig:Table:classification.
The six categories have been defined based on the analysis of the most common vulnerabilities, and how they affect the SC execution. 
Each category represents the general scope for a specific set of vulnerabilities that should be addressed in the SC development.

// #pagebreak()
#figure(
  align(center, 
    gridx(
      columns: 3,
      header-rows: 2,
      auto-vlines: false,
      column-gutter: 10pt,
      row-gutter: 10pt,
      auto-hlines: false,
      align: center + horizon,
      hlinex(stroke: black + 1pt),
      [*Code*], [*Title*], [*Summary*],
      hlinex(gutter-restrict: auto, stroke: black + 0.5pt),
      [#scv(1)], [Timestamp #linebreak() maniupulation], [Timestamp used in #linebreak() control-flow, randomness and storage, #linebreak() can open an exploit due to an ability #linebreak() for validator to manipulate the timestamp],

      hlinex(gutter-restrict: auto, stroke: black + 0.5pt),

      [#scv(2)], [Pseudo-randomness], [Using block number, block hash,#linebreak()block timestamp are not truly #linebreak() random generated parameters,#linebreak()and can be manipulated by the adversary validator],

      hlinex(gutter-restrict: auto, stroke: black + 0.5pt),
      
      [#scv(3)], [Invalidly-coded #linebreak() states], [When coding business logic,#linebreak() control-flow checks#linebreak()can be incorrectly coded resulting the SC#linebreak()entering into invalid state],

      hlinex(gutter-restrict: auto, stroke: black + 0.5pt),
      
      [#scv(4)], [Access Control#linebreak()exploits], [This is a more broad categorisation of #linebreak() vulnerabilities. It occurs when an #linebreak() adversary calls a restricted function.#linebreak()This is specifically present in#linebreak()_upgradeability_ and _deleteability_ of SCs],

      hlinex(gutter-restrict: auto, stroke: black + 0.5pt),
      
      [#scv(5)], [Arithmetic operations], [SCs are suspected to the same arithmetic #linebreak() bugs as classic programs.#linebreak()Therefore, unchecked operations can #linebreak() result in underflow/overflow or deletion by zero],

      hlinex(gutter-restrict: auto, stroke: black + 0.5pt),
      
      [#scv(6)], [Unchecked externall#linebreak()calls], [Unchecked re-entrant, forward, delegate#linebreak()calls can result in the contract#linebreak()entering into unexpected state],
      hlinex(gutter-restrict: auto, stroke: black + 1pt),
    )
  ),
  kind: "Table",
  supplement: [Table],
  caption: "Classification of SC vulnerabilities",
  placement: auto
) <Table:classification>


== Setting the scene <Section:Scene>

Even with the raised awareness for the security and safety of SCs, recent reports from _Code4rena_ still show #ref_scv(3), #ref_scv(4) and #ref_scv(5)
commonly present in the recent audit reports @arcade_report @ondo_report @centrifuge_report.

In particular, in @centrifuge_report, a relatively simple calculation mistake resulted in other SC users being unable to withdraw their funds.

It can be seen that SC Vulnerabilities illustrated in @fig:Table:classification are still evident in modern SCs, resulting in opening them up to exploits of different severity levels.
Looking at the mentioned reports, there is little consensus about the weight of each vulnerability.
Therefore, we can not classify any particular vulnerability as more severe than the other as it solely depends on the context in the code it is present.
Furthermore, it has been realised that additional tooling or alternative SCLs need to be discovered to minimise the exposure of SC code to the earlier-mentioned vulnerabilities.

#pagebreak()

= Current Solutions

== Overview

Different solutions have been presented to mitigate the consistency in the presence of vulnerabilities and programmer mistakes. 
We can generally categorise them into two groups: safe SCLs, which allow users to write safe and secure code, particularly described in @Chapter:SCL, 
and formal verification tools used alongside traditional SCLs presented in @Chapter:FVT.

This chapter reviews both categories of tools, allowing us to evaluate their effectiveness in correlation to usability,
aiming to provide a concise framework to analyse and work with the SC tools dedicated to producing
error-proof DApps. 

== Formal Verification Tools <Chapter:FVT>

Formal verification describes the assessment of the correctness of a system concerning a formal specification @eth_formal_verification. 
The specification is usually described
in terms of verifiable models using mathematical proofs. There are multiple ways to verify a program formally focused on specific parts. _Model checking_ utilises propositional logic 
to verify the mathematical abstractions of the system @model_checking. _Theorem proving_ involves verifying
relations between the model and the statements about the system @theorem_proving. Finally, _symbolic execution_ focuses
on the execution of the program using symbolic values instead of concrete values @eth_formal_verification.

KEVM #footnote[https://jellopaper.org/index.html] is a tool that provides an executable semantics of EVM using $KK$ framework. 
It uses reachability logic to reason symbolically about the system @kevm. KEVM is a powerful tool that operates at the EVM bytecode level.
Specifically, SC developers are required to write a specification in a separate file that is checked against the compiled EVM bytecode of the SC.
Whilst this provides more fine-grained assurance of the safety and correctness, it requires specialised knowledge of the $KK$ framework and EVM
semantics, hence significantly increasing the development time.

The other interesting tool is Dafny #footnote[https://dafny.org/latest/DafnyRef/DafnyRef]. Dafny is a general-purpose tool that checks inputs in any language 
using Hoare-logic and high-level annotations. Although Dafny offers compilation to some system languages, Solidity is not yet a supported target. 
Notably, work in the field suggests that the Dafny can be an effective and easy-to-use tool to produce a formal specification @dafny_deductive.
The syntax resembles a traditional imperative style and is substantially easier to learn and understand than KEVM.

Some tools can be used alongside Solidity code, such as Scribble #footnote[https://docs.scribble.codes]. 
Scribble enables developers to provide formal specifications of functions inside docstrings seamlessly integrating with existing Solidity code. 
It offers VS Code extensions and is actively maintained by Consensys #footnote[https://consensys.io/diligence/scribble]. 
The trade-off is the limited expressiveness in comparison with KEVM and Dafny.

Finally, experiments have been conducted to verify SC without any formal annotations. 
In particular, VeriSmart focuses explicitly on ensuring arithmetic safety and preciseness in SCs @so2019verismart. 
However, VeriSmart fails to detect other types of errors, 
although an effort has been made to apply the verifier to more areas of SC @azure.


Formal verification is a multi-disciplinary field offering multiple ways of reason about the systems. One of the actively researched topics
is bounded model verification @SMBC. Developers are required to reason about the programs as finite state machines (FSM)@model_fsm.
This reasoning approach is more apparent in SC development since the state transition is at the core of blockchain execution.
Bounded model checking has been realised by only a few experimental projects such as Solidifier @solidifer and Microsoft @azure.
Both projects attempt to translate Solidity code to an intermediate modelling language, Boogie @boogie. Boogie then leverages
SMT solvers to find any assertion violations.

Overall, we can see that formal verification tools provide a robust way of ensuring the safety of SCs. 
While significant effort has been made in the field, it is evident that formal verification tools in SC development
attempt to compensate for Solidity's implicit state transitions and lack of _implicit_ safety.


== Safe Smart Contract Languages <Chapter:SCL>

Multiple attempts have been made to address a flawed programming model of Solidity @sc_survey. Alternative SCLs aim to provide
built-in safety features in a type system, modelling, and function declaration to minimise the need for
external tooling. 

Some languages, such as Vyper #footnote[https://docs.vyperlang.org/en/latest/index.html], strive for simplicity.
By stripping off some low-level features, Vyper minimises the developer's chances of misusing the dangerous operations.
It also provides overflow checking, signed integers, and other safe arithmetic operations. However, Vyper is still immature, 
and the recent bug in the compiler caused a massive re-entrancy exploit in the _curve.fi_ AMM protocol @curve.
Furthermore, Vyper still suffers from the same implicit state transition problem as Solidity.

// To address the problem, it has been realised that a functional programming style may be better for SC development due to
// an explicit approach to reason about a state transition. Although some small effort has been made to adapt Haskell, neither project received any long-term support. It is still worth looking at some of the languages that suggest novice approaches to development. 

Flint is an experiment language with protected calls and asset types @flint. Protected calls introduce a role-based access system
where the SC developer can specify the permitted caller to a message function. Another unique feature is array-bounded loops
that partially address the halting problem. Flint also addresses a state-transition problem by allowing developers to specify
all possible states in the contract. The message functions need to specify the state transition, which occurs explicitly.
The language provides a significant improvement in a modelling approach. However, it still lacks the modelling SC input data in terms
of constraints and invariants, and explicit state transition is still an optional feature that the developer can miss in using.

Another promising SCL reasons about SC development through dependent and polymorphic types @idris. It extends Idris #footnote[https://www.idris-lang.org] 
and makes the developer model the SC as part of a state transition function by adopting a functional programming style. Dependent types provide a more
fine-grained control over the input and output data that flow through the SC functions. In particular, similar to Haskell, the language offers _side-effects_ 
functionality that resembles _IO_ monads in Haskell. The downside of the approach is that the syntax has become too cumbersome for other developers to learn. Thus,
it has been stated that the language does not strive for simplicity and sacrifices it for safety.

== Problem Statement

We can identify the positive trend in providing the safety of SCs.
Modern formal verification methods offer support to SC developers in ensuring that their code satisfies the requirements of the system, while
proposed SCL solutions offer runtime safety, minimising the need for the former.

However, there has been no effort to combine the two approaches into a single development process. Formal verification tools
focus on the validation of functional correctness and model consistency of a program at the compile time, whereas SCLs focus on data validation 
at the runtime. Recent work suggests that the improved optimisation of SMT solvers allows us to turn the formal model specification into 
the runtime assertions @runtime_assert. Furthermore, no effort has been made to minimise false negatives in SC formal modelling, 
even though the methods have been developed for traditional systems, such as Event-B @event_b.

#pagebreak()

= Proposed Solution <Chapter:Solution>

== Outline

In light of the above, we believe there is a need for a solution that combines two approaches to allow SC developers to reason
about their program in terms of FSM models that can be verified at the compile time for functional correctness and model consistency,
and enable an automatic generation of invariants and constraints to validate the data at runtime.

We propose _Folidity_, a safe smart contract language. Folidity will offer the model-first approach to the development process
while featuring the functional-first programming style. The language intends to offer a safe and secure-by-design approach to the programming, 
ensuring the developer is aware of any state transitions during execution.

The list of feature requirements has been comprised based on the vulnerabilities described in @Table:classification.


/ *1. Provide abstraction over timestamp* <Requirement:1>: in response to #ref_scv(1). We are interested in the limited use of timestamps in SCs in favour of block number or another safe primitive.
/ *2. Provide a safe interface for randomness* <Requirement:2>: in response to #ref_scv(2). Folidity should also provide source of randomness through a standardised interface.
/ *3. Enable model-first approach in development* <Requirement:3> :in response to #ref_scv(3). Developers should reason about the storage in terms of models and how they are updated by events. This approach is inspired by the Event-B @event_b work, which can also be applied to SC development.
/ *4. Explicit state checks at runtime* <Requirement:4>: in response to #ref_scv(3) and #ref_scv(6). Similar to #ref_req(3), SC developers should be aware of any state transitions that update the state of the model. State transitions must happen explicitly and be validated at the runtime to guarantee _liveness_.
/ *5. Static typing* <Requirements:5>: in response to #ref_scv(3) and #ref_scv(5).
/ *6. Polymorphic-dependent types* <Requirement:6>: in response to #ref_scv(3). Polymorphic-dependent types should be part of a runtime assertion check during state transition and model mutation #footnote[_Model mutation_ and _state transition_ refer to the same process. They are used interchangeably].
/ *7. Role-based access* <Requirement:7>: in response to #ref_scv(4). All message functions that mutate the model should be annotated with the role-access header specifying which set of accounts is allowed to call it.
/ *8. Checked arithmetic operations* <Requirement:8>: in response to #ref_scv(5). All arithmetic operations should be checked by default, and the developer is responsible for explicitly specifying the behaviour during over/underflow, similar to Rust.
/ *9. Enforced checked recursion or bounded loops* <Requirement:9>: in response to #ref_scv(3).
Infinite loops should not be permitted, and any loops should generally be discouraged in favour of recursion. The recursion base case should be specified explicitly with appropriate invariants.
Bounded loops may be used but should be limited to list or mapping iterations.

As part of the language design, the SC building workflow is illustrated in @Figure:compilation.

#figure(
  image("figures/compilation.png", width: 70%),
  caption: "Build workflow",
) <Figure:compilation>

As one of the core features of Folidity, formal verification is part of the build process.
Having verified the model consistency, invariants, and constraints, the program is considered safe to generate runtime assertions.

Another core feature is a pure computation context of the SC in Folidity. As illustrated in @fig:context:old,
state mutations to the contract storage and the global state (e.g. account balances) happen independently of each other. 
Folidity proposes a new execution model when a portion of a global state is _embedded_ into the local state of the SC 
as shown in @fig:context:transformed. _Global state_ refers to the overall state of the blockchain system (e.g. account balances), 
whereas _local state_ describes the storage of an individual SC.


#figure(
  image("figures/trad_context.png", width: 60%), 
  caption: "Traditional execution context", 
  ) <context:old>

#figure(
  image("figures/new_context.png", width: 60%), 
  caption: "Transformed execution context", 
  ) <context:transformed>

=== Model consistency: Simple example

As an example of the theory behind model consistency in SCs, let's look at role-based access. Suppose:

#set list(indent: 1em, tight: false, spacing: 1em)
- $* = \{ "All addresses" \}$

- $M = \{ "Moderators of the system" \}$

- $A = \{ "Admins of the system" \}$

Then, we can model a role-based access hierarchy as
$ A subset.eq M subset.eq * $

Subsequently, given some event for the system `add_mod(a: Address)`, we can define following invariants for the system:

$
i_0 := "card"(A) = 1 \
i_2 := "card"(B) = 5 
$

And the invariant for the event:

$
i_2 := c in A
$

#pagebreak()

Where
- $c$ - caller's address

- $i_n$ - enumerated invariant with some boolean statement

- $"card"(...)$ - cardinality of a set

For the denoted event, suppose we mutate the model by adding an address to a set of admins:
$ A: A union \{ a \}$

Then, we can verify the model consistency for some state transition from an initial state $S$ to a new state $S'$, $S arrow.r S'$, using propositional logic.

#v(1.5em)

$
frac(
  (i_0 and i_1 and i_2) arrow.r A union \{ a \}\, a in * \, c in A,
  A union \{ a \}
  )
$

#v(1.5em)

However, as it can be seen, one of the premises violates the invariant, in particular:

#v(1.5em)

$
frac(
  "card"(A) = 1 arrow.r A union \{ a \}\, a in *,
  A union \{ a \}
  )
$

#v(1.5em)

In practice, the following error can be picked at the compile time by using symbolic execution of the code.
The other invariant, $i_2$, can be picked at the runtime by generating an appropriate assertion.


== Implementation

The language will be implemented using Rust #footnote[https://www.rust-lang.org] due to its memory-safety guarantees and efficiency. 
Different parser-combinators alongside custom lexers are going to be used for the development of the parser. 
Folidity also requires SMT-solver for formal verification and generation of runtime assertions. In order to facilitate this functionality,
Z3#footnote[https://microsoft.github.io/z3guide] will be used since it also provides Rust bindings. It was debated to use Boogie, since it provides
a higher-level abstraction, but it was quickly discarded due to lack of documentation and increased development time.

As a target blockchain for the language, Algorand #footnote[https://developer.algorand.org] has been selected. 
Algorand is a decentralized blockchain platform designed for high-performance and low-cost transactions, 
utilising a unique consensus algorithm called Pure Proof-of-Stake to achieve scalability, security, and decentralisation @algorand.
One of the potential drawbacks of Folidity is a computational overhead due to complex abstractions and additional assertions. 
EVM-based blockchains have varying costs for the execution, i.e. fees, that depend on the complexity of a SC. 
On the contrary, although Algorand has a limited execution stack, it offers fixed, low transaction fees.
Additionally, Algorand execution context explicitly operates in terms of state transition, which perfectly suits the paradigm of Folidity.
Finally, Algorand offers opt-in functionality and local wallet storage, allowing users to explicitly opt-in to use the SC.
This provides additional support in the role-based access control in Folidity.

As a target compilation language, Tealish #footnote[https://tealish.tinyman.org] has been chosen.
Although, Algorand offers Teal #[https://developer.algorand.org/docs/get-details/dapps/avm/teal/] –
a low-level, stack-based programming language. Due to time limitations of the project, it is more realistic to use Tealish.
It operates on the same level as Teal while offering useful abstractions speeding up the development of Folidity.


== Scope

As part of the development process, it has been decided to limit the scope to supporting only a single SC execution.
Cross-contract calls require extra consideration in design and development. Therefore, #ref_scv(6) may not be fully addressed in the final report.
Additionally, optimisation of the execution is not considered relevant at this stage in favour of safety and simplicity.
Finally, Algorand offers smart signatures, a program that is delegated a signing authority #footnote[https://developer.algorand.org/docs/get-details/dapps/smart-contracts/smartsigs].
As they operate in a different way from SCs, they are also outside the scope of this project.

= Project Planning

A significant groundwork in research of current solutions and their limitations has been done as illustrated by Gannt chart in \aref{Appendix:Gannt}.
Since the requirements have been collected, some progress has been made in the design of BNF grammar that will later pave the way for the development
of the parser. It is still possible to research more formal verification methods during the grammar design. \
From the beginning of January, the first iteration of grammar should be completed, and the active development of the type checker and formal verifier should begin.


#pagebreak()

#counter(heading).update(0)
#set heading(numbering: "A.")

= Gannt Chart

#pagebreak()

#bibliography("ECS.bib", full: true)