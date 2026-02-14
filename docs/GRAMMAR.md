# ULissy Formal Grammar Specification

**Version 0.4.2**
**Extracted from the working compiler (compiler/crates/ulissy-parser)**
**February 2026**

*"A Programming Language for Moving Machines"*

---

## Notation

This grammar uses Extended Backus–Naur Form (EBNF) with the following conventions:

| Notation | Meaning |
|----------|---------|
| `'keyword'` | Terminal (literal token) |
| `IDENTIFIER` | Named token from lexer |
| `rule` | Non-terminal (grammar rule) |
| `a b` | Sequence (a followed by b) |
| `a \| b` | Alternative (a or b) |
| `[ a ]` | Optional (zero or one) |
| `{ a }` | Repetition (zero or more) |
| `( a )` | Grouping |
| `/* ... */` | Comment |

---

## 1. Program Structure

```ebnf
program         = { statement } EOF ;
```

## 2. Statements

```ebnf
statement       = identity_decl
                | let_decl
                | var_decl
                | const_decl
                | fn_decl
                | type_decl
                | enum_decl
                | config_block
                | computed_decl
                | import_stmt
                | every_block
                | when_block
                | after_block
                | send_stmt
                | if_stmt
                | if_let_stmt
                | match_stmt
                | for_stmt
                | return_stmt
                | expression_stmt ;
```

### 2.1 Declarations

```ebnf
identity_decl   = 'identity' IDENTIFIER '=' expression ;

let_decl        = 'let' IDENTIFIER [ ':' type_expr ] '=' expression ;

var_decl        = 'var' IDENTIFIER [ ':' type_expr ] [ '=' expression ] ;

const_decl      = 'const' IDENTIFIER [ ':' type_expr ] '=' expression ;
```

### 2.2 Functions

```ebnf
fn_decl         = [ 'async' ] 'fn' IDENTIFIER
                  '(' [ param_list ] ')'
                  [ '->' type_expr ]
                  [ where_clause ]
                  block ;

param_list      = parameter { ',' parameter } ;

parameter       = IDENTIFIER ':' type_expr [ '=' expression ] ;

where_clause    = 'where' expression { ',' expression } ;
```

### 2.3 Type Declarations

```ebnf
type_decl       = 'type' IDENTIFIER '{' { field_decl } { invariant } '}' ;

field_decl      = [ 'computed' ] IDENTIFIER ':' type_expr [ '=' expression ] ;

invariant       = 'invariant' expression ;
```

### 2.4 Enum Declarations

```ebnf
enum_decl       = 'enum' IDENTIFIER [ '<' type_param_list '>' ]
                  '{' enum_variant { ',' enum_variant } [ ',' ] '}' ;

type_param_list = IDENTIFIER { ',' IDENTIFIER } ;

enum_variant    = IDENTIFIER [ '(' variant_fields ')' ] [ '=' expression ] ;

variant_fields  = type_expr { ',' type_expr }
                | named_field { ',' named_field } ;

named_field     = IDENTIFIER ':' type_expr ;
```

### 2.5 Config Block

```ebnf
config_block    = 'config' '{' config_field { ',' config_field } [ ',' ] '}' ;

config_field    = IDENTIFIER ':' expression ;
```

### 2.6 Computed Properties

```ebnf
computed_decl   = 'computed' IDENTIFIER ':' type_expr computed_body ;

computed_body   = '=' expression                            /* single expression */
                | '{' object_field { ',' object_field } '}' /* object literal */ ;
```

### 2.7 Imports

```ebnf
import_stmt     = 'import' import_path [ 'as' IDENTIFIER ] ;

import_path     = IDENTIFIER { '.' IDENTIFIER } ;
```

## 3. Temporal Blocks (ULissy-Specific)

```ebnf
every_block     = 'every' expression [ 'when' expression ] block ;

when_block      = 'when' expression block ;

after_block     = 'after' expression block ;
```

## 4. Messaging (ULissy-Specific)

```ebnf
send_stmt       = 'send' 'to' expression '{'
                  send_field { ',' send_field } [ ',' ]
                  '}' ;

send_field      = IDENTIFIER ':' expression ;
```

## 5. Control Flow

### 5.1 Conditionals

```ebnf
if_stmt         = 'if' expression block [ else_branch ] ;

if_let_stmt     = 'if' 'let' IDENTIFIER '=' expression block [ else_branch ] ;

else_branch     = 'else' if_stmt
                | 'else' if_let_stmt
                | 'else' block ;
```

### 5.2 Pattern Matching

```ebnf
match_stmt      = 'match' expression '{'
                  { match_case }
                  '}' ;

match_case      = 'case' pattern [ 'where' expression ] ':' block
                | 'default' ':' block ;

pattern         = '_'                                           /* wildcard */
                | literal                                       /* value match */
                | '.' IDENTIFIER [ '(' binding_list ')' ]       /* enum variant */
                | IDENTIFIER                                    /* binding */
                | '(' pattern { ',' pattern } ')'               /* tuple */ ;

binding_list    = IDENTIFIER { ',' IDENTIFIER }
                | 'let' IDENTIFIER { ',' 'let' IDENTIFIER } ;
```

### 5.3 Loops

```ebnf
for_stmt        = 'for' IDENTIFIER 'in' expression block ;
```

### 5.4 Return

```ebnf
return_stmt     = 'return' [ expression ] ;
```

### 5.5 Expression Statement

```ebnf
expression_stmt = expression ;
```

## 6. Expressions

### 6.1 Precedence Table (lowest to highest)

| Level | Operators | Associativity | Rule |
|-------|-----------|---------------|------|
| 1 | `=` | Right | assignment |
| 2 | `??` | Right | nil coalescing |
| 3 | `\|\|` | Left | logical OR |
| 4 | `&&` | Left | logical AND |
| 5 | `==` `!=` | Left | equality |
| 6 | `<` `>` `<=` `>=` | Left | comparison |
| 7 | `+` `-` | Left | additive |
| 8 | `*` `/` `%` | Left | multiplicative |
| 9 | `-` `!` | — | unary prefix |
| 10 | `.` `?.` `()` `[]` `.unit` | Left | postfix |
| 11 | literals, identifiers, `(...)` | — | primary |

### 6.2 Expression Rules

```ebnf
expression      = assignment_expr ;

assignment_expr = nil_coalescing_expr [ '=' assignment_expr ] ;

nil_coalescing_expr
                = or_expr { '??' or_expr } ;

or_expr         = and_expr { '||' and_expr } ;

and_expr        = equality_expr { '&&' equality_expr } ;

equality_expr   = comparison_expr { ( '==' | '!=' ) comparison_expr } ;

comparison_expr = additive_expr { ( '<' | '>' | '<=' | '>=' ) additive_expr } ;

additive_expr   = multiplicative_expr { ( '+' | '-' ) multiplicative_expr } ;

multiplicative_expr
                = unary_expr { ( '*' | '/' | '%' ) unary_expr } ;

unary_expr      = ( '-' | '!' ) unary_expr
                | postfix_expr ;

postfix_expr    = primary_expr { postfix_op } ;

postfix_op      = '.' IDENTIFIER                                /* member access */
                | '.' IDENTIFIER '(' [ arg_list ] ')'           /* method call */
                | '?.' IDENTIFIER                               /* optional member */
                | '?.' IDENTIFIER '(' [ arg_list ] ')'          /* optional method */
                | '(' [ arg_list ] ')'                          /* function call */
                | '[' expression ']'                             /* index access */
                | '.' UNIT_SUFFIX ;                             /* unit value */
```

### 6.3 Primary Expressions

```ebnf
primary_expr    = literal
                | IDENTIFIER
                | handle
                | facet_address
                | facet_path
                | search_expr
                | breadcrumb_expr
                | array_literal
                | dict_literal
                | object_literal
                | interpolated_string
                | lambda_expr
                | '(' expression ')' ;
```

### 6.4 Arguments

```ebnf
arg_list        = argument { ',' argument } ;

argument        = [ IDENTIFIER ':' ] expression ;
```

## 7. Domain-Specific Expressions

### 7.1 Handles and Facets

```ebnf
handle          = '@' IDENTIFIER ;                              /* @alice */

facet_address   = IDENTIFIER '@' IDENTIFIER ;                   /* dix@alice */

facet_path      = IDENTIFIER '@' IDENTIFIER '/' IDENTIFIER ;    /* home@bob/lights */
```

### 7.2 Unit Values

```ebnf
/* Unit suffixes are parsed as postfix member access on numeric literals.
   The type checker resolves them to domain-specific types. */

UNIT_SUFFIX     = 'meters' | 'kilometers' | 'miles'            /* Distance */
                | 'seconds' | 'minutes' | 'hours' | 'days'     /* Duration */
                | 'percent' ;                                   /* Percentage */

/* Examples: 10.minutes, 500.meters, 80.percent */
```

### 7.3 Breadcrumb Constructor

```ebnf
breadcrumb_expr = 'breadcrumb' '('
                  'cell' ':' expression ','
                  'context' ':' expression ','
                  'previous' ':' expression
                  ')' ;
```

### 7.4 Search Expressions

```ebnf
search_expr     = 'search' search_target
                  [ 'where' search_filter { ',' search_filter } ]
                  [ 'ranked' 'by' search_ranking ] ;

search_target   = 'nearby' [ '(' expression ')' ]              /* spatial nearby */
                | 'within' '(' expression ',' expression ')'   /* spatial within area */
                | handle                                        /* identity lookup */
                | STRING_LITERAL ;                              /* text search */

search_filter   = 'trust' comparison_op expression
                | 'facet' '==' expression
                | 'active' 'within' expression
                | 'org' '==' expression
                | IDENTIFIER comparison_op expression ;

comparison_op   = '==' | '!=' | '>' | '<' | '>=' | '<=' ;

search_ranking  = ( 'trust' | 'distance' | 'recency' | 'relevance' )
                  ( 'asc' | 'desc' ) ;
```

## 8. Literals

```ebnf
literal         = INT_LITERAL
                | FLOAT_LITERAL
                | STRING_LITERAL
                | 'true'
                | 'false'
                | 'nil' ;

INT_LITERAL     = DECIMAL_INT | HEX_INT | BINARY_INT ;
DECIMAL_INT     = DIGIT { DIGIT } ;
HEX_INT         = '0x' HEX_DIGIT { HEX_DIGIT } ;
BINARY_INT      = '0b' ( '0' | '1' ) { '0' | '1' } ;

FLOAT_LITERAL   = DIGIT { DIGIT } '.' DIGIT { DIGIT } ;

STRING_LITERAL  = '"' { CHAR | ESCAPE } '"' ;

ESCAPE          = '\n' | '\t' | '\r' | '\\' | '\"' ;
```

### 8.1 Interpolated Strings

```ebnf
interpolated_string
                = '"' { string_part } '"' ;

string_part     = CHARS                                         /* literal text */
                | '\(' expression ')' ;                         /* interpolation */
```

### 8.2 Collection Literals

```ebnf
array_literal   = '[' [ expression { ',' expression } [ ',' ] ] ']' ;

dict_literal    = '[' dict_entry { ',' dict_entry } [ ',' ] ']' ;

dict_entry      = expression ':' expression ;

object_literal  = '{' object_field { ',' object_field } [ ',' ] '}' ;

object_field    = IDENTIFIER ':' expression                     /* explicit */
                | IDENTIFIER ;                                  /* shorthand: { x } means { x: x } */
```

### 8.3 Lambda Expressions

```ebnf
lambda_expr     = '{' [ '|' param_names '|' ] expression '}' ;

param_names     = IDENTIFIER { ',' IDENTIFIER } ;
```

## 9. Type Expressions

```ebnf
type_expr       = simple_type
                | generic_type
                | optional_type
                | function_type
                | tuple_type
                | union_type ;

simple_type     = IDENTIFIER ;
                  /* Built-in: Int, Float, String, Bool, Bytes,
                     PublicKey, Signature, Handle, TIT, SharedSecret,
                     H3Cell, Location, Distance, Coordinates,
                     Moment, Duration, Interval,
                     Hash, Ciphertext, Nonce,
                     BatteryLevel, PowerMode,
                     Uint32, Uint64 */

generic_type    = IDENTIFIER '<' type_expr { ',' type_expr } '>' ;
                  /* Examples: Array<Int>, Chain<Breadcrumb>,
                     Envelope<Message>, Optional<Handle> */

optional_type   = type_expr '?' ;
                  /* Examples: Handle?, Int?, Breadcrumb? */

function_type   = '(' [ type_expr { ',' type_expr } ] ')' '->' type_expr ;

tuple_type      = '(' type_expr ',' type_expr { ',' type_expr } ')' ;

union_type      = type_expr '|' type_expr { '|' type_expr } ;
```

## 10. Blocks

```ebnf
block           = '{' { statement } '}' ;
```

## 11. Lexical Grammar

### 11.1 Identifiers and Keywords

```ebnf
IDENTIFIER      = ALPHA { ALPHA | DIGIT | '_' } ;

ALPHA           = 'a'..'z' | 'A'..'Z' | '_' ;
DIGIT           = '0'..'9' ;
HEX_DIGIT       = DIGIT | 'a'..'f' | 'A'..'F' ;
```

### 11.2 Reserved Keywords (51)

```
identity    let         var         const       fn
type        struct      enum        trait       impl
if          else        match       case        guard
for         while       in          where       when
every       after       within      timeout     budget
send        to          from        as          with
return      throw       throws      async       await
import      export      public      private     internal
true        false       nil         self        Self
computed    invariant   config      search      default
```

### 11.3 Operators and Delimiters

```ebnf
/* Arithmetic */    +   -   *   /   %
/* Comparison */    ==  !=  <   >   <=  >=
/* Logical */       &&  ||  !
/* Assignment */    =   +=  -=  *=  /=
/* Optional */      ?   ??  ?.
/* Range */         ..  ..<
/* Arrow */         ->  =>
/* Delimiters */    (   )   {   }   [   ]   ,   :   ;   .   @
```

### 11.4 Comments

```ebnf
line_comment    = '//' { ANY_CHAR } NEWLINE ;

block_comment   = '/*' { ANY_CHAR | block_comment } '*/' ;
                  /* Block comments nest */
```

### 11.5 Whitespace

```ebnf
WHITESPACE      = ' ' | '\t' | '\r' | '\n' ;
                  /* Whitespace is insignificant; statement separation
                     is determined by syntactic context, not newlines. */
```

---

## Appendix A: Grammar Ambiguities and Resolutions

### A.1 Unit Suffix vs. Member Access

The expression `10.minutes` could parse as either:
- Unit value: `UnitValue(10, "minutes")`
- Member access: `Member(10, "minutes")`

**Resolution:** The parser treats `NUMBER '.' IDENTIFIER` as a unit value when
the identifier is a known unit suffix (`meters`, `kilometers`, `miles`,
`seconds`, `minutes`, `hours`, `days`, `percent`). All other cases are member
access. The type checker enforces that unit values produce the correct
domain-specific type.

### A.2 Object Literal vs. Block

The token `{` could begin either an object literal or a block.

**Resolution:** The parser looks ahead: if the pattern is `'{' IDENTIFIER ':'`
it parses as an object literal. Otherwise, it parses as a block. This means
single-statement blocks like `{ x }` are parsed as blocks, and explicit
object notation `{ x: x }` is required for single-field objects.

### A.3 Enum Variant vs. Member Access

The expression `.human` could be:
- Enum variant shorthand: `EntityType.human`
- Member access on an implicit object

**Resolution:** Leading `.IDENTIFIER` is parsed as an enum variant shorthand
(similar to Swift). The type checker infers the enum type from context.

### A.4 Handle vs. At-Operator

The token `@` followed by an identifier is always a handle literal (`@alice`).
The bare `@` token is not used as an operator.

### A.5 Facet Address vs. Email-Like Syntax

The expression `dix@alice` is parsed as a facet address by the lexer.
The lexer recognizes `IDENTIFIER '@' IDENTIFIER` as a single `FacetAddress`
token when the characters are contiguous (no spaces).

### A.6 If Statement vs. If-Let

Both begin with the `if` keyword. The parser disambiguates by checking
whether the next two tokens are `'let' IDENTIFIER '='`, which signals
an `if_let_stmt`. Otherwise, it's a standard `if_stmt`.

---

## Appendix B: Example Parse Trees

### B.1 Breadcrumb Collection

```ulissy
every 10.minutes when battery > 20 {
    let crumb = breadcrumb(
        cell: here.h3(7),
        context: sensors.digest,
        previous: me.trajectory.last?.hash ?? "genesis"
    ).signed(me)
    me.trajectory.append(crumb)
}
```

**Parse tree:**
```
EveryBlock
├── interval: UnitValue(Int(10), "minutes")
├── condition: Binary(Member(Ident("battery")), Gt, Int(20))
└── body:
    ├── LetDecl "crumb"
    │   └── init: MethodCall
    │       ├── object: Breadcrumb
    │       │   ├── cell: MethodCall(Ident("here"), "h3", [Int(7)])
    │       │   ├── context: Member(Ident("sensors"), "digest")
    │       │   └── previous: NilCoalescing
    │       │       ├── primary: OptionalMember
    │       │       │   ├── object: Member(Member(Ident("me"), "trajectory"), "last")
    │       │       │   └── member: "hash"
    │       │       └── fallback: String("genesis")
    │       ├── method: "signed"
    │       └── args: [Ident("me")]
    └── ExprStmt: MethodCall
        ├── object: Member(Member(Ident("me"), "trajectory"))
        ├── method: "append"
        └── args: [Ident("crumb")]
```

### B.2 Search with Filters

```ulissy
let nearby = search nearby
             where trust > 0.5, active within 1.hours
             ranked by recency desc
```

**Parse tree:**
```
LetDecl "nearby"
└── init: Search
    ├── target: Nearby(radius: None)
    ├── filters:
    │   ├── TrustThreshold(Greater, Float(0.5))
    │   └── ActiveWithin(UnitValue(Int(1), "hours"))
    └── ranking: Recency(Descending)
```

### B.3 Pattern Matching

```ulissy
match facet {
    case .dix(let handle):
        showBroadcast(handle)
    case .pay(let handle):
        showPayment(handle)
    default:
        print("Unknown facet")
}
```

**Parse tree:**
```
MatchStmt
├── subject: Ident("facet")
└── cases:
    ├── Case
    │   ├── pattern: EnumVariant("dix", bindings: ["handle"])
    │   └── body: [Call(Ident("showBroadcast"), [Ident("handle")])]
    ├── Case
    │   ├── pattern: EnumVariant("pay", bindings: ["handle"])
    │   └── body: [Call(Ident("showPayment"), [Ident("handle")])]
    └── Default
        └── body: [Call(Ident("print"), [String("Unknown facet")])]
```

---

## Appendix C: Language Evolution Notes

### C.1 Constructs Under Consideration

The following constructs appear in the whitepaper but are not yet
implemented in the parser:

| Construct | Whitepaper Example | Status |
|-----------|-------------------|--------|
| `guard ... else` | `guard count >= 100 else { return }` | Planned |
| `within ... timeout` | `within 10.seconds { ... } timeout { ... }` | Planned |
| `budget ... exceeded` | `budget 5% battery { ... } exceeded { ... }` | Planned |
| `with powerMode:` | `with powerMode: .performance { ... }` | Planned |
| `trait` / `impl` | Trait-based polymorphism | Planned |
| `struct` | Distinct from `type` | Reserved keyword, not differentiated |

### C.2 Version History

| Version | Grammar Changes |
|---------|----------------|
| v0.1.0 | Initial: 14 statement types, core expressions |
| v0.2.0 | Added `config` block, `computed` properties |
| v0.3.0 | Added `for` loops, `if let`, `default` case |
| v0.3.1 | `self`/`Self`/`config` as identifiers in expressions |
| v0.4.2 | `search` expressions (4 targets, 5 filter types, 4 rankings) |

---

**Document Version:** 0.4.2
**Grammar Rules:** 68
**Statement Types:** 18
**Expression Types:** 26
**Keywords:** 51
**Authors:** GNS Foundation
**License:** MIT

---

*"The journey is the proof."*
— ULissy Design Principle
