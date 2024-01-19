#let author_linked(
  name: "Author name",
  email: none,
 ) = {
  if email == none {
    text(name)
  } else {
    let email = "mailto:" + email
    link(email)[#name]
  }
}

#let page_style(body: none) = {
    set page(
    numbering: "1",
    margin: (inside: 1.5in, outside: 1.0in, top: 0.6in, bottom: 0.8in),
  )
  set align(left)
  show par: it => [
    #set par(justify: true)
    #pad(top: 0.3em, bottom: 0.3em, it)
  ]

  show heading.where(level: 1): it => [
    #set text(size: 24pt, weight: "semibold")
    #pagebreak()
    #pad(top: 5em, bottom: 2em, it.body)
  ]


  [ #body ]
}

#let cover(
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
) = {
  let body-font = "New Computer Modern"
  let sans-font = "New Computer Modern Sans"

  set document(title: title, author: author.at("name"))
  set page(
    numbering: none,
    margin: (inside: 1.5in, outside: 1.0in, top: 0.6in, bottom: 0.8in),
  )
  counter(page).update(0)

  set text(
    font: body-font, 
    size: 12pt, 
    lang: "en",
  )
  set align(center)

  v(9em) 
  par()[
      #text(14pt, "Electronics and Computer Science") \
      #text(14pt, "Faculty of Engineering and Physical Sciences") \
      #text(14pt, "University of Southampton")
  ]

  v(6.5em)

  let author_content = author_linked(name: author.at("name"), email: author.at("email"))

  box(width: 250pt, height: 89pt)[
    #grid(
      columns: 1,
      gutter: 2em,
      text(author_content),
      text(date),
      box(text(14pt, weight: "bold", title))
    )
  ]

  let supervisor_content = author_linked(name: supervisor.at("name"), email: supervisor.at("email"))
  let examiner_content = author_linked(name: examiner.at("name"), email: examiner.at("email"))

  v(15em)
  par()[
    #text("Project supervisor: ") #supervisor_content \
    #text("Second examiner: " ) #examiner_content
  ]


  let award_text = if is_progress_report {
    "A project progress report submitted for the award of"
   } else {
    "A project report submitted for the award of" 
   }

  v(4.3em)
  par()[
    #text(14pt, award_text) \
    #text(14pt, program)
  ]
  

}

#let abstract(
  author: (
    name: "Author name",
    email: none,
  ),
  program: "Program name",
  is_progress_report: false,
  content: lorem(150),
) = {
  set page(
    numbering: none,
    margin: (inside: 1.5in, outside: 1.0in, top: 0.6in, bottom: 0.8in),
  )
  counter(page).update(1)

  let body-font = "New Computer Modern"
  let sans-font = "New Computer Modern Sans"
  set text(
    font: body-font, 
    size: 12pt, 
    lang: "en",
  )
  
  set align(center)

  v(9.5em)
  text("UNIVESITY OF SOUTHAMPTON")

  v(0.5em)
  underline(text("ABSTRACT"))

  v(0.5em)
  par()[
    #text("FACULTY OF ENGINEERING AND PHYSICAL SCIENCES") \
    #text("ELECTRONICS AND COMPUTER SCINCE")
  ]

  v(0.5em)
  let report_text = if is_progress_report {
    "A project progress report submitted for the award of"
   } else {
    "A project report submitted for the award of" 
   }
  let award_text = report_text + " " + program
  underline(text(award_text))

  v(0.5em)
  let author_content = author_linked(name: author.at("name"), email: author.at("email"))
  text("By " + author_content)

  v(2em)
  set par(linebreaks: "optimized", justify: true)
  text(content)
}

#let originality_statement(
  acknowledged: "I have acknowledged all sources, and identified any content taken from elsewhere.",
  resources: "I have not used any resources produced by anyone else.",
  foreign: "I did all the work myself, or with my allocated group, and have not helped anyone else",
  material: "The material in the report is genuine, and I have included all my data/code/designs.",
  reuse: "I have not submitted any part of this work for another assessment.",
  participants: "My work did not involve human participants, their cells or data, or animals."
) = {
  let box(info: none) = block(stroke: 0.5pt + black, width: 100%, pad(4pt, text(weight: "bold", info)))
  page_style(body: [ 
    #let body-font = "New Computer Modern"
    #let sans-font = "New Computer Modern Sans"
    #set text(
      font: body-font, 
      size: 12pt, 
      lang: "en",
    )

    #set page(
      numbering: "i",
      margin: (inside: 1.5in, outside: 1.0in, top: 0.6in, bottom: 1.5in),
    )
    = Statement of Originality

    #set list(tight: false, spacing: 10pt)

    - I have read and understood the #link("http://ecs.gg/ai")[ECS Academic Integrity information] and the University’s #link("https://www.southampton.ac.uk/quality/assessment/academic_integrity.page")[Academic Integrity Guidance for Students].
    - I am aware that failure to act in accordance with the #link("http://www.calendar.soton.ac.uk/sectionIV/academic-integrity-regs.html")[Regulations Governing Academic Integrity] may lead to the imposition of penalties which, for the most serious cases, may include termination of programme.
    - I consent to the University copying and distributing any or all of my work in any form and using third parties (who may be based outside the EU/EEA) to verify whether my work contains plagiarised material, and for quality assurance purposes.

    #v(1em)

    *You must change the statements in the boxes if you do not agree with them.*


    We expect you to acknowledge all sources of information (e.g. ideas, algorithms, data) using citations. You must also put quotation marks around any sections of text that you have copied without paraphrasing. If any figures or tables have been taken or modified from another source, you must explain this in the caption and cite the original source.

    #box(info: acknowledged)

    If you have used any code (e.g. open-source code), reference designs, or similar resources 
    that have been produced by anyone else, you must list them in the box below. In the
    report, you must explain what was used and how it relates to the work you have done.

    #box(info: resources)

    You can consult with module teaching staff/demonstrators, but you should not show
    anyone else your work (this includes uploading your work to publicly-accessible repositories 
    e.g. Github, unless expressly permitted by the module leader), or help them to
    do theirs. For individual assignments, we expect you to work on your own. For group
    assignments, we expect that you work only with your allocated group. You must get
    permission in writing from the module teaching staff before you seek outside assistance,
    e.g. a proofreading service, and declare it here.

    #box(info: foreign)

    We expect that you have not fabricated, modified or distorted any data, evidence, 
    references, experimental results, or other material used or presented in the report. 
    You must clearly describe your experiments and how the results were obtained, and include
    all data, source code and/or designs (either in the report, or submitted as a separate
    file) so that your results could be reproduced.

    #box(info: material)

    We expect that you have not previously submitted any part of this work for another
    assessment. You must get permission in writing from the module teaching staff before
    re-using any of your previously submitted work for this assessment.

    #box(info: reuse)

    If your work involved research/studies (including surveys) on human participants, their
    cells or data, or on animals, you must have been granted ethical approval before the
    work was carried out, and any experiments must have followed these requirements. You
    must give details of this in the report, and list the ethical approval reference number(s)
    in the box below.

    #box(info: participants)
  ])
}