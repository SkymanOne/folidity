# eBNF grammar

Based on the [sample program](samples.md) we can design the first version of eBNF grammar

```bnf
program    := <metadata>
metadata   := <version> <author>
version    := `version:` `"` <int> `.` <int> `.` <int>
author     := `author:` `"` <string> `<` <string> `>`
```