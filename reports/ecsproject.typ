#let author_linked(
  name: "Author name",
  email: none
 ) = {
  if email == none {
    text(name)
  } else {
    let email = "mailto:" + email
    link(email)[
      #{
         name
      }
     ]
  }
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
    margin: (left: 1.5in, right: 1in, top: 0.6in, bottom: 0.8in),
  )
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

  box(width: 240pt, height: 89pt)[
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

  v(15.5em)
  par()[
    #text("Project supervisor: ") #{ supervisor_content }\
    #text("Second examiner: " ) #{ examiner_content }
  ]


  let award_text = if is_progress_report {
    "A project progress report submitted for the award of"
   } else {
    "A project report submitted for the award of" 
   }

  v(3.3em)
  par()[
    #text(14pt, award_text) \
    #text(14pt, program)
  ]
  

}
