#import "ecsproject.typ": *

// Use everthning together

#show: doc => use_project(
  title: "My project",
  author: (
    name: "Author name",
    email: none
  ),
  supervisor: (
    name: "Supervisor name",
    email: none
  ),
  examiner: (
    name: "Supervisor name",
    email: none
  ),
  date: "December 22, 2023",
  program: "BSc Computer Science",
  is_progress_report: false,
  originality_statements: (
    acknowledged: "I have acknowledged all sources, and identified any content taken from elsewhere.",
    resources: "I have not used any resources produced by anyone else.",
    foreign: "I did all the work myself, or with my allocated group, and have not helped anyone else",
    material: "The material in the report is genuine, and I have included all my data/code/designs.",
    reuse: "I have not submitted any part of this work for another assessment.",
    participants: "My work did not involve human participants, their cells or data, or animals."
  ),
  abstract_text: lorem(50),
  acknowledgments_text: lorem(50),
  doc
)


= Proposed Solution

== Outline

#lorem(1500)

= Hey

#lorem(500)

== Subhey

#lorem(500)



#pagebreak()

#bibliography("ECS.bib", full: true)