# eBNF grammar

Based on the [sample program](samples.md) we can design the first version of eBNF grammar

## Grammar
Based on holistic approach

```xml
<program>      := <metadata> <decl>+
<metadata>     := (<version> <author>) | (<author> <version>)
<version>      := `version` `:` `"` <number> `.` <number> `.` <number>
<author>       := `author` `:` `"` <ident> `<` <ident> `>`

<decl>         := <func_decl> | <model_decl> | <state_decl> | <enum_decl> | <struct_decl>

<func_decl>    :=  `@init`? <attrs>+ <vis> `fn` <type_decl> <ident> `(` <params>? `)` <state_bound>? <st_block>? `{` <func_body> `}`
<type_decl>    := <type> | `(` <param> `)`

<attrs>        := `@` `(` <attr_ident> `)`
<attr_ident>   := <ident> | ( <expr> `|` )*
<params>       := <param> | (<params> `,`)*
<param>        := <ident> `:` <type>
<vis>          := `pub` | `view` `(` <state_param> `)`
<state_bound>  := `when` <state_param> <arr> <state_param>
<func_body>    := (<statement>)*
<state_param>  := (<ident> <ident>?) | `()`

<st_block>     := `st` <st_body>
<st_body>      := <expr> | `{` <st_list> `}`
<st_list>      := <expr> | (<st_list> `,`)*

<statement>    := <var> | <assign> | <if> | <for> | <foreach> | <return> | <func_call> | <state_t>
<var>          := let `mut`? <var_ident> (`:` <type>)? (`=` <expr>)?
<var_ident>    := (<ident> | <decon>)
<decon>        := `{` <decon_list> `}`
<decon_list>   := <ident> | (<decon_list> `,` )*

<assign>       := <ident> `=` <expr>
<if>           := `if` `(` <expr> `)` `{` <statement> `}` (`else` `{` <statement> `}`)?
<foreach>      := `for` `(` `var_ident` `in` (<ident> | <range>) `)` `{` <statement> `}`
<for>          := `for` `(` <var> `;` <expr> `;` <expr> `)` `{` <statement> `}`
<range>        := `range` `(` <number> `to` <number> `)` 
<return>       := `return` <expr>
<state_t>      := <ident> `{` <struct_args> `}`
<struct_args>  := <expr> | (<struct_args> `,`)* | <arg_obj>
<struct_arg>   := <ident> `:` <expr>
<arg_obj>      := `..` <ident>

<model_decl>   := `model` <ident> `{` params `}` <st_block>?

<state_decl>   := `state` <ident> (`from` <ident> <ident>)?  <state_body> <st_block>?
<state_body>   := `(` <ident> `)` |  `{` params `}`
<enum_decl>    := `enum` `{` <ident>+ `}`
<struct_decl>  := `struct` `{` params `}`

<type>         := `int` | `uint` | `float` | `char` | `string` | `hex` 
             | `address` | `()` | `bool` | <set_type> | <list_type> | <mapping_type>


<set_type>     := `Set` `<` <type> `>`
<list_type>    := `List` `<` <type> `>`
<mapping_type> := `Mapping` `<` <type> <mapping_rel> <type> `>`
<mapping_rel>  := (`>`)? `-` (`/`)? (`>`)? `>`

<char>         := ? '` <char>* `'`
<hex>          := `hex` `"` <char>* `"`
<address>      := `a` `"` <char>* `"`

<digit>        := [0-9]
<number>       := <digit>+

<bool>         := `true` | `false`
<rel>          := `==` | `!=` | `<` | `>` | `<=` | `>=` | `in` 
<bool_op>      := `||` | '&&'

<period>       := `.`
<float>        := <number> <period> <number>?

<func_pipe>    := <expr> (`:>` <func_call>)+
<member_acc>   := <expr> (`.` <ident>)+
<func_call>    := <ident> `(` <args>? `)`
<args>         := <expr> | (<args> `,`)*


<plus>         := `+`
<minus>        := `-`
<div>          := `/`
<mul>          := `*`
<not>          := `!`
<modulo>       := `%`
<expr>         := <not>? <expr_nested>
<expr_nested>  := <expr> <bool_op> <expr>
<cond>         := <expr> <rel> <expr> 
<math_expr>    := <term> ( (<plus> | <minus>) <term> )*
<term>         := <factor> ( (<mul> | <div> | <modulo>) <factor> )*
<factor>       := <ident> | <constant> | <func_call> | <func_pipe> | <member_acc> | `(` <expr> `)`
<constant>     := <number> | <float> | <bool> | <string> | <hex> | <address>
<ident>        := <char>+
<arr>          := `->`

```

## Legend:
- `<ident>` - eBNF element
- `?` - optional element
- `(  )` - grouping
-  `+` - one or more
-  `*` - zero or more
-  \`ident\` - literal token