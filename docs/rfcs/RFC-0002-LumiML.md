# RFC-0002: LumiML Document Specification

**Category:** Format Standard  
**Status:** Draft  

---

## 1. Abstract
LumiML is a lightweight, human-readable document markup language designed for zero-overhead native rendering.

## 2. Element Grammar
```lumiml
page {
    title "Title Text"
    heading { text "Heading" }
    paragraph { text "Paragraph text" }
    button { text "Label" goto "lumi://domain.lumi" }
    container { ... }
    row { ... }
    column { ... }
    divider {}
    codeblock { text "code" }
    badge { text "badge text" }
}
```
