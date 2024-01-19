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

#let margins = (inside: 1.5in, outside: 1.0in, top: 1.0in, bottom: 1.0in)

#let page_style(
  page_numbering: "1",
  title_numbering: "1.",
  doc
) = {

  set page(
    numbering: page_numbering,
    margin: margins,
  )
  let body-font = "New Computer Modern"
  let sans-font = "New Computer Modern Sans"
  set text(
    font: body-font, 
    size: 12pt, 
    lang: "en",
  )
  set align(left)
  show par: it => [
    #set par(justify: true)
    #pad(top: 0.3em, bottom: 0.3em, it)
  ]

  set heading(numbering: title_numbering)
  show heading.where(level: 1): it => {
    set text(size: 26pt, weight: "semibold")
    let counter_display = if it.numbering != none {
      counter(heading).display(it.numbering)
    }
    pad(top: 1.4em, bottom: 1.4em, [
      #counter_display #it.body
    ])
  }

  show heading.where(level: 2): it => {
    set text(size: 18pt, weight: "semibold")
    let counter_display = if it.numbering != none {
      counter(heading).display(it.numbering)
    }
    pad(top: 0.5em, bottom: 0.5em, [
      #counter_display #it.body
    ])
  }

  show heading.where(level: 3): it => {
    set text(size: 14pt, weight: "semibold")
    let counter_display = if it.numbering != none {
      counter(heading).display(it.numbering)
    }
    pad(top: 0.5em, bottom: 0.5em, [
      #counter_display #it.body
    ])
  }

  set page(header: locate(loc => {
    let current_page = counter(page).at(loc).at(0)
    let before_elems = query(
      heading.where(level: 1).before(loc),
      loc,
    )
    let after_elems = query(
      heading.where(level: 1).after(loc),
      loc,
    )
    let heading_present = false;
    if after_elems != () {
      let h_counter = counter(page).at(after_elems.first().location()).at(0)
      if current_page == h_counter {
        heading_present = true
      }
    }
    if not heading_present  {
      if before_elems != () {
        let h_counter = counter(heading.where(level: 1)).display()
        let last_header = before_elems.last().body
        emph(h_counter + last_header) + h(1fr) + counter(page).display(page_numbering)
        v(-1em)
        line(stroke: 0.5pt, length: 100%)
      }
    }
  }), footer: locate(loc => {
    let current_page = counter(page).at(loc).at(0)
    let before_elems = query(
      heading.where(level: 1).before(loc),
      loc,
    )
    let after_elems = query(
      heading.where(level: 1).after(loc),
      loc,
    )
    let heading_present = false;
    if before_elems != () {
      let h_counter = counter(page).at(before_elems.last().location()).at(0)
      if current_page == h_counter {
        heading_present = true
      }
    }
    if heading_present {
      let p_counter = counter(page).display(page_numbering)
      align(center, p_counter)
    }
  }))

  doc
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
    margin: margins,
  )
  counter(page).update(0)

  set text(
    font: body-font, 
    size: 12pt, 
    lang: "en",
  )
  set align(center)

  v(7em) 
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
    margin: margins,
  )
  counter(page).update(0)
  
  set align(center)
  let body-font = "New Computer Modern"
  let sans-font = "New Computer Modern Sans"
  set text(
      font: body-font, 
      size: 12pt, 
      lang: "en",
  )

  v(8.5em)
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
  page_style(
    page_numbering: "i",
    [ 

    #heading(level: 1, numbering: none, "Statement of Originality")

    #set list(tight: false, spacing: 10pt, indent: 1.5em)

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

#let table_of_contents() = {
  page_style(
    page_numbering: "i",
    {
      set page(header: [])
      heading(level: 1, "Contents", numbering: none)
      
      show outline.entry.where(level: 1): it => {
          v(1.2em, weak: true)
          strong(it)
      }
      outline(title: none, indent: 2em, fill: line(length: 100%, stroke: (thickness: 1pt, dash: "loosely-dotted")))
    }
  )
}

#let acknowledgments(text: lorem(100)) = {
    page_style(
    page_numbering: "i",
    {
      set page(header: [])
      heading(level: 1, "Acknowledgments", numbering: none)
      
      text
    }
  ) 
}


#let use_project(
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
  originality_statement: (
    acknowledged: "I have acknowledged all sources, and identified any content taken from elsewhere.",
    resources: "I have not used any resources produced by anyone else.",
    foreign: "I did all the work myself, or with my allocated group, and have not helped anyone else",
    material: "The material in the report is genuine, and I have included all my data/code/designs.",
    reuse: "I have not submitted any part of this work for another assessment.",
    participants: "My work did not involve human participants, their cells or data, or animals."
  ),
  abstract_text: lorem(50),
  acknowledgments_text: lorem(50),
  page_numbering: "1",
  title_numbering: "1.",

  doc
) = {
  cover(
    title: title,
    supervisor: supervisor,
    examiner: examiner,
    author: author,
    date: date,
    program: program,
    is_progress_report: is_progress_report
  )

  abstract(
    author: author,
    program: program,
    is_progress_report: is_progress_report,
    content: abstract_text
  )

  acknowledgments(text: acknowledgments_text)

  table_of_contents()

  show: doc => page_style(
    page_numbering: page_numbering,
    title_numbering: title_numbering,
  doc
  )

  doc
}