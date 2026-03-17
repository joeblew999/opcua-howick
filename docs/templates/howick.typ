// Howick document template — clean modern style
#let howick-doc(
  title: "",
  body
) = {
  set document(title: title)
  set page(
    paper: "a4",
    margin: (top: 2.5cm, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
    header: [
      #set text(size: 8pt, fill: rgb("#999999"))
      #grid(
        columns: (1fr, 1fr),
        align(left)[#title],
        align(right)[#context datetime.today().display("[day] [month repr:long] [year]")],
      )
      #line(length: 100%, stroke: 0.5pt + rgb("#eeeeee"))
    ],
    footer: [
      #line(length: 100%, stroke: 0.5pt + rgb("#eeeeee"))
      #set text(size: 8pt, fill: rgb("#999999"))
      #grid(
        columns: (1fr, 1fr),
        align(left)[ubuntu Software — Gerard Webb],
        align(right)[#context counter(page).display()],
      )
    ],
  )

  // Body text
  set text(
    font: "Helvetica Neue",
    size: 10pt,
    fill: rgb("#1a1a1a"),
  )
  set par(leading: 0.75em, spacing: 1.2em)

  // Headings
  show heading.where(level: 1): it => {
    v(1.5em)
    block[
      #text(size: 20pt, weight: 700, fill: rgb("#0f172a"))[#it.body]
      #v(0.2em)
      #line(length: 100%, stroke: 1.5pt + rgb("#3b82f6"))
    ]
    v(0.5em)
  }

  show heading.where(level: 2): it => {
    v(1em)
    text(size: 13pt, weight: 600, fill: rgb("#1e3a5f"))[#it.body]
    v(0.3em)
  }

  show heading.where(level: 3): it => {
    v(0.8em)
    text(size: 11pt, weight: 600, fill: rgb("#374151"))[#it.body]
    v(0.2em)
  }

  // Tables
  show table: it => {
    set text(size: 9pt)
    set table(
      stroke: none,
      fill: (x, y) => if y == 0 { rgb("#0f172a") } else if calc.odd(y) { rgb("#f8fafc") } else { white },
    )
    show table.cell.where(y: 0): set text(fill: white, weight: 600)
    it
  }

  // Code blocks
  show raw.where(block: true): it => {
    block(
      fill: rgb("#f1f5f9"),
      radius: 4pt,
      inset: 12pt,
      width: 100%,
    )[#text(font: "Menlo", size: 8.5pt, fill: rgb("#334155"))[#it]]
  }

  show raw.where(block: false): it => {
    box(
      fill: rgb("#f1f5f9"),
      radius: 3pt,
      inset: (x: 4pt, y: 2pt),
    )[#text(font: "Menlo", size: 8.5pt, fill: rgb("#334155"))[#it]]
  }

  // Title block
  if title != "" {
    v(1cm)
    text(size: 28pt, weight: 700, fill: rgb("#0f172a"))[#title]
    v(0.3em)
    line(length: 6cm, stroke: 2pt + rgb("#3b82f6"))
    v(0.6em)
    text(size: 9pt, fill: rgb("#64748b"))[ubuntu Software — Gerard Webb]
    v(2em)
  }

  body
}

// Pandoc-generated elements
#let horizontalrule = line(length: 100%, stroke: 0.5pt + rgb("#e2e8f0"))
#let authors = ()

// Main entry — pandoc calls this
#show: howick-doc.with(title: "$title$")
$body$
