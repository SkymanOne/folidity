# eBNF grammar

Based on the [sample program](samples.md) we can design the first version of eBNF grammar

## Grammar
Based on holistic approach

```xml
<program>      := <decl>+

<decl>         := <func_decl> | <model_decl> | <state_decl> | <enum_decl> | <struct_decl>

<func_decl>    :=  `@init`? <attrs>+ <view>? `fn` <type_decl> <ident> `(` <params>? `)` <state_bound>? <st_block>? `{` <func_body> `}`
<type_decl>    := <type> | `(` <param> `)`

<attrs>        := `@` `(` <attr_ident> `)`
<attr_ident>   := <ident> | ( <expr> `|` )*
<params>       := <param> | <param> (`,` <params>)*
<param>        := <ident> `:` <type>
<view>          := `view` `(` <state_param> `)`
<state_bound>  := `when` <state_param> <arr> ( <state_param> | <state_param> (`,` <state_param>)*)
<func_body>    := (<statement>)*
<state_param>  := (<ident> <ident>?) | `()`

<st_block>     := `st` <expr>

<statement>    := <var> | <assign> | <if> | <for> | <foreach> | <return> | <func_call> | <state_t> `;`
<state_t>      := `move` <struct_init>
<var>          := let `mut`? <var_ident> (`:` <type>)? (`=` <expr>)?
<var_ident>    := (<ident> | <decon>)
<decon>        := `{` <decon_list> `}`
<decon_list>   := <ident> | <ident> (`,` <decon_list>)*

<assign>       := <ident> `=` <expr>
<if>           := `if` <expr> ` `{` <statement> `}` (`else` <if>? )*
<foreach>      := `for` `(` <var_ident> `in` (<ident> | <range>) `)` `{` <statement> `}`
<for>          := `for` `(` <var> `;` <expr> `;` <expr> `)` `{` <statement> `}`
<return>       := `return` <expr>
<struct_init>  := <ident> : `{` <struct_args> `}`
<struct_args>  := <expr> | (`,` <expr>)* | <arg_obj>
<struct_arg>   := <ident> `:` <expr>
<arg_obj>      := `|` `..` <ident>

<model_decl>   := `model` <ident> `{` params `}` <st_block>?

<state_decl>   := `state` <ident> (`from` <ident> <ident>)?  <state_body> <st_block>?
<state_body>   := `(` <ident> `)` |  `{` params `}`
<enum_decl>    := `enum` `{` (<ident> | <ident> (`,` <ident>)* ) `}`
<struct_decl>  := `struct` `{` <params> `}`

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
<expr_nested>  := <term> <bool_op> <expr>
<cond>         := <expr> <rel> <expr> 
<math_expr>    := <term> ( (<plus> | <minus>) <term> )*
<term>         := <factor> ( (<mul> | <div> | <modulo>) <factor> )*
<factor>       := <ident> | <constant> | <func_call> | <func_pipe> | <member_acc> | `(` <expr> `)`
<constant>     := <number> | <float> | <bool> | <string> | <hex> | <address> | <list>
<list>         := `[` ( <expr>? | <expr> (`,` <expr>)* ) `]`
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