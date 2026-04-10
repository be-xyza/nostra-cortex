# Potluck: Dynamic documents as personal software
By Ink & Switch
Source: https://www.inkandswitch.com/potluck/

## Introduction and Motivation
Potluck explores the idea of bridging the gap between freeform documents (like notes) and rigid applications. Applications force data into specific structures and taxonomies, while documents are flexible, permissive, and versatile but static. 

The proposed solution is **gradual enrichment**: allowing users to record information in natural, messy ways (text) and slowly adding formal structure and computational behavior only as needed. Spreadsheets are a key inspiration here, where users start with a freeform grid and add formulas iteratively.

## Potluck: An Environment for Dynamic Documents
Potluck acts as a note-taking application where users can gradually turn text documents into interactive software through a loop of: extracting data, computing with it, and displaying results back in the text.

### 1. Extracting data with searches
Users define custom patterns to detect data within the text. Searches can be simple literal strings (e.g., `11 g`) or generalized patterns (e.g., `{number} g`). These searches can reference built-in or custom patterns to pull out entities via named capture groups (`{number:amount}`). Advanced searches can use regular expressions, but are encapsulated in reusable named blocks.

### 2. Running live computations
Once data is extracted into a table (similar to a spreadsheet), users can run small JavaScript expressions (computed properties) on the extracted data. These formulas re-evaluate reactively. Higher-level functions allow for common tasks like summing up numbers. 

### 3. Dynamic annotations
Computed values can be displayed in the original text document as annotations. These annotations can insert new text, restyle original text, or inject interactive widgets (like a slider to scale recipe servings or a countdown timer for a duration). 

### 4. Reusing searches
Potluck allows users to reuse existing searches (like `duration`, extracting time and displaying a countdown widget) that others have made. This ecosystem of reusable patterns prevents users from having to build everything from scratch.

### 5. Spatial queries
To capture relationships not easily expressed in regular inline patterns (e.g., associating an ingredient in directions with its quantity in the ingredients list above), Potluck supports spatial queries. Users can search for elements based on their position relative to other elements in the document, like "the first ingredient with a matching name before this point."

## Findings & Philosophy
- **Versatility**: The freeform nature of text makes Potluck adaptable to tracking expenses, recipes, workout logs, agendas, etc.
- **State in text**: Application state lives natively in the text (e.g., writing `[x]` or `08/31/2022`). There is no hidden metadata. If you copy/paste the text, the state comes with it, bringing the affordances of text editors (undo/redo, copy/paste) for free.
- **Tool composition**: Tools (computations/patterns) can be reused in different contexts or combined (e.g., having a trip agenda and an expense tracker in the exact same freeform document).
- **Challenges**: Parsing fuzzy, pre-existing text data is highly challenging. However, for personal notes, people naturally invent "micro-syntaxes" and adjust their typing to fit the recognizers. Structured data views (like a calendar rendering text dates) were experimented with to overcome the 1D layout limitation of text.

## Future Work
- **Machine Learning**: ML / LLMs could be integrated to interpret unstructured text more robustly (e.g., matching different terms for the same ingredient) or to help end-users write search queries and computations via natural language. 
- **Structured data views**: Allowing users to create complex visual layouts outside the text document natively, driven by extracted text data.

## Conclusion
Information can be organically recorded and gradually enriched. The tool fits the workflow rather than the user's workflow fitting the tool. By treating text as a substrate for a user interface, users can build personal software tailored exactly to their needs.
